use bulk_merge::config::{AppConfig, LibgenDumpKind};
use bulk_merge::db::{Db, ImportRunStatus};
use bulk_merge::libgen::offline::{
    convert_dump_to_tsv, load_tsv_into_postgres, load_tsv_into_postgres_for_test_interrupt_after_staged_tables,
};
use std::fs;

fn base_config() -> AppConfig {
    let mut cfg = AppConfig::load("config/bulk-merge.toml").expect("load config");
    cfg.execution.batch.max_rows = 1000;
    cfg.execution.batch.max_bytes = 1_000_000;
    cfg.execution.memory_hard_limit_bytes = 64 * 1024 * 1024;
    cfg.progress.log_interval_seconds = 3600;
    cfg.libgen.raw.enabled = false;
    cfg.postgres.indexing.concurrent = false;
    cfg
}

fn write_tmp_dump(name: &str, contents: &str) -> String {
    let path = format!("tmp/{name}");
    fs::write(&path, contents).expect("write dump");
    path
}

#[tokio::test]
async fn libgen_offline_load_staging_swap_resumes_after_interrupt() {
    let mut cfg = base_config();
    let run_suffix = uuid::Uuid::new_v4().to_string();
    cfg.postgres.table_prefix = Some(format!("o_{run_suffix}_"));

    let dump = r#"
CREATE TABLE `fiction` (
  `ID` int(11) NOT NULL,
  `Title` varchar(255) DEFAULT NULL,
  PRIMARY KEY (`ID`)
);
INSERT INTO `fiction` VALUES (1,'a'),(2,'b');
"#;

    let dump_path = write_tmp_dump(&format!("libgen_offline_dump_{run_suffix}.sql"), dump);
    let out_dir = format!(".cache/bulk-merge/test-offline-{run_suffix}");
    let _ = fs::remove_dir_all(&out_dir);

    convert_dump_to_tsv(&cfg, LibgenDumpKind::Fiction, &dump_path, &out_dir).expect("convert to tsv");

    let db = Db::connect(&cfg).await.expect("connect");
    db.migrate().await.expect("migrate");

    let run_id = db
        .create_import_run(
            "libgen",
            &format!("offline_ds_{run_suffix}"),
            Some("v1"),
            ImportRunStatus::InProgress,
            LibgenDumpKind::Fiction,
            &dump_path,
            &cfg,
        )
        .await
        .expect("create import run");

    // Simulate an interruption after staging 1 table.
    let err = load_tsv_into_postgres_for_test_interrupt_after_staged_tables(&db, &cfg, &out_dir, run_id, 1)
        .await
        .expect_err("expected intentional interrupt");
    assert!(
        err.to_string().contains("intentional interrupt"),
        "unexpected error: {err:?}"
    );

    // Resume should complete without reloading staged data and swap into live schema.
    load_tsv_into_postgres(&db, &cfg, &out_dir, run_id)
        .await
        .expect("resume load");

    let pg_table = format!(
        "{}{}fiction",
        cfg.postgres.table_prefix.as_deref().unwrap_or(""),
        cfg.libgen.tables.fiction_prefix
    );
    let count = db
        .table_row_count(&cfg.postgres.schema_libgen, &pg_table)
        .await
        .expect("row count");
    assert_eq!(count, 2);

    db.finish_import_run(run_id, ImportRunStatus::Succeeded)
        .await
        .expect("finish run");

    let _ = fs::remove_dir_all(&out_dir);
}

