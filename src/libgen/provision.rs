use crate::config::{AppConfig, LibgenDumpKind};
use crate::db::Db;
use crate::libgen::mysql_dump::{parse_create_table, read_statements, table_prefix_for_kind, TableDef};
use anyhow::Context as _;
use tracing::{info, instrument};

#[instrument(skip_all, fields(kind = ?kind, dump = %dump_path))]
pub async fn provision_tables_from_dump(
    db: &Db,
    config: &AppConfig,
    kind: LibgenDumpKind,
    dump_path: &str,
) -> anyhow::Result<Vec<TableDef>> {
    let file = std::fs::File::open(dump_path)
        .with_context(|| format!("failed to open dump file `{dump_path}`"))?;

    info!("reading statements (create table discovery)");
    let statements = read_statements(file, config.libgen.dump.max_statement_bytes)
        .context("failed reading SQL statements from dump")?;

    let prefix = table_prefix_for_kind(
        kind,
        &config.libgen.tables.fiction_prefix,
        &config.libgen.tables.compact_prefix,
    );
    let overall_prefix = config.postgres.table_prefix.as_deref().unwrap_or("");

    let mut defs: Vec<TableDef> = Vec::new();
    for stmt in &statements {
        if let Some(def) = parse_create_table(stmt).context("failed parsing CREATE TABLE")? {
            defs.push(def);
        }
    }

    if defs.is_empty() {
        anyhow::bail!("no CREATE TABLE statements found in dump");
    }

    info!(tables = defs.len(), "provisioning postgres tables from mysql schema");
    for def in &defs {
        let pg_table = format!("{overall_prefix}{prefix}{}", def.name);
        db.create_table_from_def(&config.postgres.schema_libgen, &pg_table, def)
            .await
            .with_context(|| format!("failed creating table `{}`", pg_table))?;
    }

    Ok(defs)
}
