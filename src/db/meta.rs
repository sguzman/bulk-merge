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

    #[instrument(skip_all, fields(schema = schema, table = table, rows = rows.len()))]
    pub async fn insert_rows_text(
        &self,
        schema: &str,
        table: &str,
        columns: &[String],
        rows: &[Vec<Option<String>>],
    ) -> anyhow::Result<()> {
        if rows.is_empty() {
            return Ok(());
        }
        let expected_cols = columns.len();
        for r in rows {
            if r.len() != expected_cols {
                anyhow::bail!(
                    "row length mismatch for insert: expected {expected_cols}, got {}",
                    r.len()
                );
            }
        }

        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let cols_sql = columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ");

        let mut qb = sqlx::QueryBuilder::new(format!(
            "insert into {schema_q}.{table_q} ({cols_sql}) "
        ));
        qb.push_values(rows, |mut b, row| {
            for val in row {
                b.push_bind(val);
            }
        });

        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(import_run_id = import_run_id, key = %checkpoint_key))]
    pub async fn set_checkpoint_offset(
        &self,
        import_run_id: i64,
        checkpoint_key: &str,
        offset: u64,
    ) -> anyhow::Result<()> {
        let value = serde_json::json!({ "offset": offset });
        sqlx::query(
            r#"
insert into bm_meta.import_checkpoint (import_run_id, checkpoint_key, checkpoint_value)
values ($1, $2, $3)
on conflict (import_run_id, checkpoint_key)
do update set checkpoint_value = excluded.checkpoint_value, updated_at = now()
"#,
        )
        .bind(import_run_id)
        .bind(checkpoint_key)
        .bind(value)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip_all, fields(import_run_id = import_run_id, key = %checkpoint_key))]
    pub async fn get_checkpoint_offset(
        &self,
        import_run_id: i64,
        checkpoint_key: &str,
    ) -> anyhow::Result<Option<u64>> {
        let rec: Option<(serde_json::Value,)> = sqlx::query_as(
            r#"
select checkpoint_value
from bm_meta.import_checkpoint
where import_run_id = $1 and checkpoint_key = $2
"#,
        )
        .bind(import_run_id)
        .bind(checkpoint_key)
        .fetch_optional(&self.pool)
        .await?;

        let Some((value,)) = rec else {
            return Ok(None);
        };
        let offset = value
            .get("offset")
            .and_then(|v| v.as_u64())
            .unwrap_or(0);
        Ok(Some(offset))
    }
}

fn quote_ident(ident: &str) -> String {
    format!("\"{}\"", ident.replace('\"', "\"\""))
}

fn map_mysql_type_to_postgres(mysql: &str) -> &'static str {
    // Phase 1 policy: map fields 1-to-1 and prioritize ingest robustness/speed.
    // We store everything as text for now; later phases can add typed/normalized views.
    let _ = mysql;
    "text"
}
