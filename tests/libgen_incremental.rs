use bulk_merge::config::{AppConfig, LibgenDumpKind};
use bulk_merge::db::{Db, ImportRunStatus};
use bulk_merge::libgen::ingest::{ingest_dump_rows, IngestMode, IngestPlan};
use bulk_merge::libgen::provision::provision_tables_from_dump;
use std::fs;

fn base_config() -> AppConfig {
    let mut cfg = AppConfig::load("config/bulk-merge.toml").expect("load config");
    // Make tests deterministic and avoid huge buffers.
    cfg.execution.batch.max_rows = 1000;
    cfg.execution.batch.max_bytes = 1_000_000;
    cfg.execution.memory_hard_limit_bytes = 64 * 1024 * 1024;
    cfg.progress.log_interval_seconds = 3600;
    cfg.libgen.raw.enabled = false;
    cfg
}

fn write_tmp_dump(name: &str, contents: &str) -> String {
    let path = format!("tmp/{name}");
    fs::write(&path, contents).expect("write dump");
    path
}

#[tokio::test]
async fn libgen_update_upserts_and_deletes() {
    let mut cfg = base_config();
    let run_suffix = uuid::Uuid::new_v4().to_string();
    cfg.postgres.table_prefix = Some(format!("t_{run_suffix}_"));

    // Ensure PK columns are configured.
    cfg.libgen.incremental.primary_key_columns.fiction = vec!["ID".to_string()];
    cfg.libgen.incremental.apply_deletes = true;

    let db = Db::connect(&cfg).await.expect("connect");
    db.migrate().await.expect("migrate");

    let dump_v1 = r#"
CREATE TABLE `fiction` (
  `ID` int(11) NOT NULL,
  `Title` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`ID`)
);
INSERT INTO `fiction` VALUES (1,'old'),(2,'bye');
"#;
    let dump_v2 = r#"
CREATE TABLE `fiction` (
  `ID` int(11) NOT NULL,
  `Title` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`ID`)
);
INSERT INTO `fiction` VALUES (1,'new'),(3,'hi');
"#;

    let path_v1 = write_tmp_dump(&format!("libgen_test_v1_{run_suffix}.sql"), dump_v1);
    let path_v2 = write_tmp_dump(&format!("libgen_test_v2_{run_suffix}.sql"), dump_v2);

    // INGEST v1
    let ingest_run_id = db
        .create_import_run(
            "libgen",
            &format!("ds_{run_suffix}"),
            Some("v1"),
            ImportRunStatus::InProgress,
            LibgenDumpKind::Fiction,
            &path_v1,
            &cfg,
        )
        .await
        .expect("create import run v1");

    let defs = provision_tables_from_dump(&db, &cfg, LibgenDumpKind::Fiction, &path_v1, ingest_run_id)
        .await
        .expect("provision v1");

    let plan = IngestPlan {
        kind: LibgenDumpKind::Fiction,
        dump_path: path_v1.clone(),
        table_defs: defs,
        overall_prefix: cfg.postgres.table_prefix.clone().unwrap_or_default(),
        kind_prefix: cfg.libgen.tables.fiction_prefix.clone(),
        mode: IngestMode::Ingest,
        conflict_columns: vec!["ID".to_string()],
        apply_deletes: false,
    };

    ingest_dump_rows(&db, &cfg, &plan, ingest_run_id)
        .await
        .expect("ingest v1");
    db.finish_import_run(ingest_run_id, ImportRunStatus::Succeeded)
        .await
        .expect("finish v1");
    db.upsert_dataset_state(
        "libgen",
        &format!("ds_{run_suffix}"),
        "fiction",
        ingest_run_id,
        Some("v1"),
    )
    .await
    .expect("dataset_state v1");

    // UPDATE v2 (upsert + deletes)
    let update_run_id = db
        .create_import_run(
            "libgen",
            &format!("ds_{run_suffix}"),
            Some("v2"),
            ImportRunStatus::InProgress,
            LibgenDumpKind::Fiction,
            &path_v2,
            &cfg,
        )
        .await
        .expect("create import run v2");

    let defs2 = provision_tables_from_dump(&db, &cfg, LibgenDumpKind::Fiction, &path_v2, update_run_id)
        .await
        .expect("provision v2");

    let plan2 = IngestPlan {
        kind: LibgenDumpKind::Fiction,
        dump_path: path_v2.clone(),
        table_defs: defs2,
        overall_prefix: cfg.postgres.table_prefix.clone().unwrap_or_default(),
        kind_prefix: cfg.libgen.tables.fiction_prefix.clone(),
        mode: IngestMode::Update,
        conflict_columns: vec!["ID".to_string()],
        apply_deletes: true,
    };

    // Update mode requires a unique constraint/index for ON CONFLICT.
    let pg_table = plan2.pg_table_for_mysql("fiction");
    db.ensure_unique_index(
        &cfg.postgres.schema_libgen,
        &pg_table,
        &plan2.conflict_columns,
        false,
    )
    .await
    .expect("ensure unique index");

    ingest_dump_rows(&db, &cfg, &plan2, update_run_id)
        .await
        .expect("update v2 ingest");

    // Apply deletes for table `fiction`.
    let deleted = db
        .delete_rows_not_seen(
            &cfg.postgres.schema_libgen,
            &pg_table,
            "fiction",
            "ID",
            update_run_id,
        )
        .await
        .expect("apply deletes");
    assert!(deleted >= 1, "expected at least one deletion");

    db.finish_import_run(update_run_id, ImportRunStatus::Succeeded)
        .await
        .expect("finish v2");
    db.upsert_dataset_state(
        "libgen",
        &format!("ds_{run_suffix}"),
        "fiction",
        update_run_id,
        Some("v2"),
    )
    .await
    .expect("dataset_state v2");

    // Validate: ID=1 updated, ID=2 deleted, ID=3 inserted.
    let title1 = db
        .get_text_by_pk(&cfg.postgres.schema_libgen, &pg_table, "ID", "1", "Title")
        .await
        .expect("get title1")
        .expect("title1 present");
    assert_eq!(title1, "new");

    let title2 = db
        .get_text_by_pk(&cfg.postgres.schema_libgen, &pg_table, "ID", "2", "Title")
        .await
        .expect("get title2");
    assert!(title2.is_none(), "expected id=2 deleted");

    let title3 = db
        .get_text_by_pk(&cfg.postgres.schema_libgen, &pg_table, "ID", "3", "Title")
        .await
        .expect("get title3")
        .expect("title3 present");
    assert_eq!(title3, "hi");

    // Dataset state should point to v2.
    let state = db
        .get_dataset_state("libgen", &format!("ds_{run_suffix}"), "fiction")
        .await
        .expect("get dataset state")
        .expect("state exists");
    assert_eq!(state.0, Some(update_run_id));
    assert_eq!(state.1.as_deref(), Some("v2"));
}
