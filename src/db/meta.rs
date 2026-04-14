use crate::config::{AppConfig, LibgenDumpKind};
use anyhow::Context as _;
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tracing::{info, instrument};

pub struct Db {
    pool: PgPool,
}

#[derive(Debug, Clone, Copy)]
pub enum ImportRunStatus {
    Pending,
    InProgress,
    Failed,
    Succeeded,
}

impl ImportRunStatus {
    fn as_str(self) -> &'static str {
        match self {
            ImportRunStatus::Pending => "pending",
            ImportRunStatus::InProgress => "in_progress",
            ImportRunStatus::Failed => "failed",
            ImportRunStatus::Succeeded => "succeeded",
        }
    }
}

impl Db {
    #[instrument(skip_all)]
    pub async fn connect(config: &AppConfig) -> anyhow::Result<Self> {
        let acquire_timeout = Duration::from_millis(config.postgres.pool.acquire_timeout_ms);
        let url = config.postgres.connection_url()?;
        let pool = PgPoolOptions::new()
            .max_connections(config.postgres.pool.max_connections)
            .min_connections(config.postgres.pool.min_connections)
            .acquire_timeout(acquire_timeout)
            .connect(&url)
            .await
            .context("failed to connect to postgres")?;

        Ok(Self { pool })
    }

    #[instrument(skip_all)]
    pub async fn migrate(&self) -> anyhow::Result<()> {
        info!("running migrations");
        sqlx::migrate!("./migrations").run(&self.pool).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(source_name = source_name, dataset_id = dataset_id, kind = ?kind))]
    pub async fn create_import_run(
        &self,
        source_name: &str,
        dataset_id: &str,
        dataset_version: Option<&str>,
        status: ImportRunStatus,
        kind: LibgenDumpKind,
        dump: &str,
        config: &AppConfig,
    ) -> anyhow::Result<i64> {
        let config_json = serde_json::json!({
            "postgres": {
                "schema_meta": config.postgres.schema_meta,
                "schema_libgen": config.postgres.schema_libgen,
            },
            "libgen": {
                "kind": format!("{kind:?}").to_lowercase(),
                "dump": dump,
            }
        });

        let rec: (i64,) = sqlx::query_as(
            r#"
insert into bm_meta.import_run (source_name, dataset_id, dataset_version, status, config_json)
values ($1, $2, $3, $4, $5)
returning id
"#,
        )
        .bind(source_name)
        .bind(dataset_id)
        .bind(dataset_version)
        .bind(status.as_str())
        .bind(config_json)
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.0)
    }

    #[instrument(skip_all, fields(import_run_id = import_run_id, status = status.as_str()))]
    pub async fn finish_import_run(
        &self,
        import_run_id: i64,
        status: ImportRunStatus,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
update bm_meta.import_run
set finished_at = now(), status = $2
where id = $1
"#,
        )
        .bind(import_run_id)
        .bind(status.as_str())
        .execute(&self.pool)
        .await?;

        Ok(())
    }

    #[instrument(skip_all, fields(schema = schema, table = table))]
    pub async fn create_table_from_def(
        &self,
        schema: &str,
        table: &str,
        def: &crate::libgen::mysql_dump::TableDef,
    ) -> anyhow::Result<()> {
        let mut cols_sql: Vec<String> = Vec::with_capacity(def.columns.len());
        for col in &def.columns {
            let col_name = quote_ident(&col.name);
            let ty = map_mysql_type_to_postgres(&col.mysql_type);
            cols_sql.push(format!("{col_name} {ty}"));
        }

        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let sql = format!(
            "create table if not exists {schema_q}.{table_q} (\n    {}\n)",
            cols_sql.join(",\n    ")
        );

        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }
}

fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('\"', "\"\""))
}

fn map_mysql_type_to_postgres(mysql: &str) -> &'static str {
    let t = mysql.trim().to_ascii_lowercase();
    let base = t.split('(').next().unwrap_or(&t);

    match base {
        "tinyint" => "smallint",
        "smallint" => "smallint",
        "mediumint" => "integer",
        "int" | "integer" => "integer",
        "bigint" => "bigint",
        "float" => "real",
        "double" => "double precision",
        "decimal" | "numeric" => "numeric",
        "char" | "varchar" => "text",
        "tinytext" | "text" | "mediumtext" | "longtext" => "text",
        "date" => "date",
        "datetime" => "timestamp",
        "timestamp" => "timestamp",
        "blob" | "tinyblob" | "mediumblob" | "longblob" => "bytea",
        _ => "text",
    }
}
