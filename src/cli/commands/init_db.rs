use crate::config::AppConfig;
use crate::db::Db;
use crate::libgen::provision::discover_table_defs_from_dump;
use crate::libgen::mysql_dump::table_prefix_for_kind;
use crate::config::LibgenDumpKind;
use tracing::{info, instrument};

#[instrument(skip_all)]
pub async fn run(config: &AppConfig) -> anyhow::Result<()> {
    info!("connecting to postgres");
    let db = Db::connect(config).await?;
    db.migrate().await?;

    if config.libgen.init.provision_tables {
        let overall_prefix = config.postgres.table_prefix.as_deref().unwrap_or("");

        // We don't create an import_run for init-db provisioning; use a sentinel id for raw storage (disabled by default in practice).
        let sentinel_import_run_id: i64 = -1;

        if let Some(path) = config.libgen.init.dumps.fiction.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
            let defs = discover_table_defs_from_dump(config, path, sentinel_import_run_id, None).await?;
            let prefix = table_prefix_for_kind(
                LibgenDumpKind::Fiction,
                &config.libgen.tables.fiction_prefix,
                &config.libgen.tables.compact_prefix,
            );
            for def in &defs {
                let pg_table = format!("{overall_prefix}{prefix}{}", def.name);
                db.create_table_from_def(
                    &config.postgres.schema_libgen,
                    &pg_table,
                    def,
                    config.libgen.incremental.strategy == "row_hash" && config.libgen.incremental.row_hash.enabled,
                    config.libgen.typing.mode,
                )
                .await?;
            }
            info!(tables = defs.len(), "provisioned fiction tables during init-db");
        }

        if let Some(path) = config.libgen.init.dumps.compact.as_deref().map(str::trim).filter(|p| !p.is_empty()) {
            let defs = discover_table_defs_from_dump(config, path, sentinel_import_run_id, None).await?;
            let prefix = table_prefix_for_kind(
                LibgenDumpKind::Compact,
                &config.libgen.tables.fiction_prefix,
                &config.libgen.tables.compact_prefix,
            );
            for def in &defs {
                let pg_table = format!("{overall_prefix}{prefix}{}", def.name);
                db.create_table_from_def(
                    &config.postgres.schema_libgen,
                    &pg_table,
                    def,
                    config.libgen.incremental.strategy == "row_hash" && config.libgen.incremental.row_hash.enabled,
                    config.libgen.typing.mode,
                )
                .await?;
            }
            info!(tables = defs.len(), "provisioned compact tables during init-db");
        }
    }

    info!("database initialized");
    Ok(())
}
