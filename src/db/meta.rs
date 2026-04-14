use crate::config::{AppConfig, LibgenDumpKind};
use anyhow::Context as _;
use sha2::{Digest as _, Sha256};
use sqlx::{postgres::PgPoolOptions, PgPool};
use std::time::Duration;
use tokio::io::AsyncReadExt;
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
        let mut opts = PgPoolOptions::new();
        if let Some(timeout_ms) = config.postgres.statement_timeout_ms {
            opts = opts.after_connect(move |conn, _meta| {
                Box::pin(async move {
                    let sql = format!("set statement_timeout = {timeout_ms}");
                    sqlx::query(&sql).execute(conn).await?;
                    Ok(())
                })
            });
        }

        let pool = opts
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

    #[instrument(skip_all, fields(import_run_id = import_run_id, path = %path))]
    pub async fn upsert_import_file(
        &self,
        import_run_id: i64,
        path: &str,
        size_bytes: Option<i64>,
        status: &str,
    ) -> anyhow::Result<i64> {
        let rec: (i64,) = sqlx::query_as(
            r#"
insert into bm_meta.import_file (import_run_id, path, size_bytes, status)
values ($1, $2, $3, $4)
on conflict (import_run_id, path)
do update set
  size_bytes = coalesce(excluded.size_bytes, bm_meta.import_file.size_bytes),
  status = excluded.status
returning id
"#,
        )
        .bind(import_run_id)
        .bind(path)
        .bind(size_bytes)
        .bind(status)
        .fetch_one(&self.pool)
        .await?;

        Ok(rec.0)
    }

    #[instrument(skip_all, fields(import_file_id = import_file_id, offset = offset, seen = records_seen, loaded = records_loaded))]
    pub async fn update_import_file_progress(
        &self,
        import_file_id: i64,
        offset: i64,
        records_seen: i64,
        records_loaded: i64,
        status: &str,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
update bm_meta.import_file
set last_offset = $2,
    records_seen = $3,
    records_loaded = $4,
    status = $5
where id = $1
"#,
        )
        .bind(import_file_id)
        .bind(offset)
        .bind(records_seen)
        .bind(records_loaded)
        .bind(status)
        .execute(&self.pool)
        .await?;
        Ok(())
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
        include_row_hash: bool,
    ) -> anyhow::Result<()> {
        let mut cols_sql: Vec<String> = Vec::with_capacity(def.columns.len());
        for col in &def.columns {
            let col_name = quote_ident(&col.name);
            let ty = map_mysql_type_to_postgres(&col.mysql_type);
            cols_sql.push(format!("{col_name} {ty}"));
        }
        if include_row_hash {
            cols_sql.push(format!("{} text not null", quote_ident("_bm_row_hash")));
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

    #[instrument(skip_all, fields(schema = schema, table = table, rows = rows.len()))]
    pub async fn insert_rows_text_on_conflict_do_nothing(
        &self,
        schema: &str,
        table: &str,
        columns: &[String],
        conflict_columns: &[String],
        rows: &[Vec<Option<String>>],
    ) -> anyhow::Result<()> {
        if rows.is_empty() {
            return Ok(());
        }
        if conflict_columns.is_empty() {
            anyhow::bail!("conflict_columns must not be empty");
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
        for c in conflict_columns {
            if !columns.iter().any(|x| x == c) {
                anyhow::bail!("conflict column `{c}` not present in columns list");
            }
        }

        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let cols_sql = columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ");
        let conflict_sql = conflict_columns
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
        qb.push(format!(" on conflict ({conflict_sql}) do nothing"));

        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(schema = schema, table = table, rows = rows.len()))]
    pub async fn copy_rows_text_tsv(
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
                    "row length mismatch for copy: expected {expected_cols}, got {}",
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

        // Use CSV format with tab delimiter. NULL is the literal \N (unquoted).
        // We quote all non-NULL fields, escaping quotes by doubling.
        let copy_stmt = format!(
            "copy {schema_q}.{table_q} ({cols_sql}) from stdin with (format csv, delimiter E'\\t', null '\\\\N', quote '\"', escape '\"')"
        );

        let mut conn = self.pool.acquire().await?;
        let mut copy_in = conn.copy_in_raw(&copy_stmt).await?;

        let mut buf: Vec<u8> = Vec::new();
        buf.reserve(rows.len().min(1024) * 128);

        for row in rows {
            for (i, val) in row.iter().enumerate() {
                if i > 0 {
                    buf.push(b'\t');
                }
                match val {
                    None => buf.extend_from_slice(b"\\N"),
                    Some(s) => {
                        buf.push(b'"');
                        for ch in s.chars() {
                            if ch == '"' {
                                buf.push(b'"');
                                buf.push(b'"');
                            } else {
                                let mut tmp = [0u8; 4];
                                buf.extend_from_slice(ch.encode_utf8(&mut tmp).as_bytes());
                            }
                        }
                        buf.push(b'"');
                    }
                }
            }
            buf.push(b'\n');
        }

        copy_in.send(buf).await?;
        copy_in.finish().await?;
        Ok(())
    }

    #[instrument(skip_all, fields(schema = schema, table = table, path = %path.display()))]
    pub async fn copy_in_tsv_file(
        &self,
        schema: &str,
        table: &str,
        columns: &[String],
        path: &std::path::Path,
        file_send_chunk_bytes: u64,
    ) -> anyhow::Result<()> {
        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let cols_sql = columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ");

        let copy_stmt = format!(
            "copy {schema_q}.{table_q} ({cols_sql}) from stdin with (format csv, delimiter E'\\t', null '\\\\N', quote '\"', escape '\"')"
        );

        let mut f = tokio::fs::File::open(path)
            .await
            .with_context(|| format!("failed to open tsv file `{}`", path.display()))?;

        let chunk_size: usize = usize::try_from(file_send_chunk_bytes.max(1))
            .unwrap_or(1_048_576)
            .max(1);
        let mut buf: Vec<u8> = vec![0u8; chunk_size];

        let mut conn = self.pool.acquire().await?;
        let mut copy_in = conn.copy_in_raw(&copy_stmt).await?;

        loop {
            let n = f.read(&mut buf).await?;
            if n == 0 {
                break;
            }
            copy_in.send(buf[..n].to_vec()).await?;
        }

        copy_in.finish().await?;
        Ok(())
    }

    #[instrument(skip_all, fields(schema = schema, table = table, rows = rows.len()))]
    pub async fn upsert_rows_text(
        &self,
        schema: &str,
        table: &str,
        columns: &[String],
        conflict_columns: &[String],
        rows: &[Vec<Option<String>>],
    ) -> anyhow::Result<()> {
        if rows.is_empty() {
            return Ok(());
        }
        if conflict_columns.is_empty() {
            anyhow::bail!("conflict_columns must not be empty for upsert");
        }

        let expected_cols = columns.len();
        for r in rows {
            if r.len() != expected_cols {
                anyhow::bail!(
                    "row length mismatch for upsert: expected {expected_cols}, got {}",
                    r.len()
                );
            }
        }

        for c in conflict_columns {
            if !columns.iter().any(|x| x == c) {
                anyhow::bail!("conflict column `{c}` not present in columns list");
            }
        }

        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let cols_sql = columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ");
        let conflict_sql = conflict_columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ");
        let update_sql = columns
            .iter()
            .map(|c| {
                let qc = quote_ident(c);
                format!("{qc} = excluded.{qc}")
            })
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
        qb.push(format!(
            " on conflict ({conflict_sql}) do update set {update_sql}"
        ));

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

    #[instrument(skip_all, fields(schema = schema, table = table, column = column, concurrent = concurrent))]
    pub async fn ensure_btree_index(
        &self,
        schema: &str,
        table: &str,
        column: &str,
        concurrent: bool,
    ) -> anyhow::Result<()> {
        let index_name = format!("idx_{}_{}", table, column);
        if concurrent {
            if self.index_exists(schema, &index_name).await.unwrap_or(false) {
                return Ok(());
            }
            let schema_q = quote_ident(schema);
            let table_q = quote_ident(table);
            let col_q = quote_ident(column);
            let idx_q = quote_ident(&index_name);
            let sql = format!(
                "create index concurrently {idx_q} on {schema_q}.{table_q} ({col_q})"
            );
            sqlx::query(&sql).execute(&self.pool).await?;
            return Ok(());
        }

        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let col_q = quote_ident(column);
        let idx_q = quote_ident(&index_name);
        let sql = format!(
            "create index if not exists {idx_q} on {schema_q}.{table_q} ({col_q})"
        );
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(schema = schema, table = table, columns = columns.len(), concurrent = concurrent))]
    pub async fn ensure_unique_index(
        &self,
        schema: &str,
        table: &str,
        columns: &[String],
        concurrent: bool,
    ) -> anyhow::Result<()> {
        if columns.is_empty() {
            anyhow::bail!("unique index columns must not be empty");
        }
        let index_name = format!("uidx_{}_{}", table, columns.join("_"));

        if concurrent {
            if self.index_exists(schema, &index_name).await.unwrap_or(false) {
                return Ok(());
            }
            let schema_q = quote_ident(schema);
            let table_q = quote_ident(table);
            let cols_sql = columns
                .iter()
                .map(|c| quote_ident(c))
                .collect::<Vec<_>>()
                .join(", ");
            let idx_q = quote_ident(&index_name);
            let sql = format!(
                "create unique index concurrently {idx_q} on {schema_q}.{table_q} ({cols_sql})"
            );
            sqlx::query(&sql).execute(&self.pool).await?;
            return Ok(());
        }

        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let cols_sql = columns
            .iter()
            .map(|c| quote_ident(c))
            .collect::<Vec<_>>()
            .join(", ");
        let idx_q = quote_ident(&index_name);
        let sql = format!(
            "create unique index if not exists {idx_q} on {schema_q}.{table_q} ({cols_sql})"
        );
        sqlx::query(&sql).execute(&self.pool).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(schema = schema, index = index))]
    pub async fn index_exists(&self, schema: &str, index: &str) -> anyhow::Result<bool> {
        let rec: Option<(i64,)> = sqlx::query_as(
            r#"
select 1
from pg_class c
join pg_namespace n on n.oid = c.relnamespace
where c.relkind = 'i' and n.nspname = $1 and c.relname = $2
"#,
        )
        .bind(schema)
        .bind(index)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec.is_some())
    }

    #[instrument(skip_all, fields(import_run_id = import_run_id, offset_end = byte_offset_end, kind = stmt_kind, table = mysql_table))]
    pub async fn insert_libgen_raw_statement(
        &self,
        import_run_id: i64,
        byte_offset_end: i64,
        stmt_kind: &str,
        mysql_table: Option<&str>,
        payload: &str,
    ) -> anyhow::Result<()> {
        let mut hasher = Sha256::new();
        hasher.update(payload.as_bytes());
        let hash = hasher.finalize().to_vec();

        sqlx::query(
            r#"
insert into src_libgen.raw_statement (import_run_id, byte_offset_end, stmt_kind, mysql_table, sha256, payload)
values ($1, $2, $3, $4, $5, $6)
on conflict (import_run_id, byte_offset_end) do nothing
"#,
        )
        .bind(import_run_id)
        .bind(byte_offset_end)
        .bind(stmt_kind)
        .bind(mysql_table)
        .bind(hash)
        .bind(payload)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip_all)]
    pub async fn list_tables_with_prefix(
        &self,
        schema: &str,
        prefix: &str,
    ) -> anyhow::Result<Vec<String>> {
        let recs: Vec<(String,)> = sqlx::query_as(
            r#"
select table_name
from information_schema.tables
where table_schema = $1 and table_type = 'BASE TABLE' and table_name like $2
order by table_name
"#,
        )
        .bind(schema)
        .bind(format!("{prefix}%"))
        .fetch_all(&self.pool)
        .await?;
        Ok(recs.into_iter().map(|r| r.0).collect())
    }

    #[instrument(skip_all, fields(schema = schema, table = table))]
    pub async fn table_row_count(&self, schema: &str, table: &str) -> anyhow::Result<i64> {
        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let sql = format!("select count(*)::bigint from {schema_q}.{table_q}");
        let rec: (i64,) = sqlx::query_as(&sql).fetch_one(&self.pool).await?;
        Ok(rec.0)
    }

    #[instrument(skip_all, fields(schema = schema, table = table, limit = limit))]
    pub async fn sample_table(
        &self,
        schema: &str,
        table: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<serde_json::Value>> {
        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let sql = format!(
            "select row_to_json(t) from (select * from {schema_q}.{table_q} limit {limit}) t"
        );
        let recs: Vec<(serde_json::Value,)> = sqlx::query_as(&sql).fetch_all(&self.pool).await?;
        Ok(recs.into_iter().map(|r| r.0).collect())
    }

    #[instrument(skip_all)]
    pub async fn recent_import_runs(
        &self,
        source_name: &str,
        limit: u32,
    ) -> anyhow::Result<Vec<(i64, String, Option<String>, String, chrono::DateTime<chrono::Utc>)>> {
        let recs: Vec<(i64, String, Option<String>, String, chrono::DateTime<chrono::Utc>)> =
            sqlx::query_as(
                r#"
select id, dataset_id, dataset_version, status, started_at
from bm_meta.import_run
where source_name = $1
order by started_at desc
limit $2
"#,
            )
            .bind(source_name)
            .bind(limit as i64)
            .fetch_all(&self.pool)
            .await?;
        Ok(recs)
    }

    #[instrument(skip_all, fields(import_run_id = import_run_id))]
    pub async fn raw_statement_count(&self, import_run_id: i64) -> anyhow::Result<i64> {
        let rec: (i64,) = sqlx::query_as(
            r#"
select count(*)::bigint
from src_libgen.raw_statement
where import_run_id = $1
"#,
        )
        .bind(import_run_id)
        .fetch_one(&self.pool)
        .await?;
        Ok(rec.0)
    }

    #[instrument(skip_all, fields(source_name = source_name, dataset_id = dataset_id, kind = kind, run_id = import_run_id))]
    pub async fn upsert_dataset_state(
        &self,
        source_name: &str,
        dataset_id: &str,
        kind: &str,
        import_run_id: i64,
        dataset_version: Option<&str>,
    ) -> anyhow::Result<()> {
        sqlx::query(
            r#"
insert into bm_meta.dataset_state
  (source_name, dataset_id, kind, last_succeeded_import_run_id, last_dataset_version)
values ($1, $2, $3, $4, $5)
on conflict (source_name, dataset_id, kind)
do update set
  last_succeeded_import_run_id = excluded.last_succeeded_import_run_id,
  last_dataset_version = excluded.last_dataset_version,
  updated_at = now()
"#,
        )
        .bind(source_name)
        .bind(dataset_id)
        .bind(kind)
        .bind(import_run_id)
        .bind(dataset_version)
        .execute(&self.pool)
        .await?;
        Ok(())
    }

    #[instrument(skip_all, fields(source_name = source_name, dataset_id = dataset_id, kind = kind))]
    pub async fn get_dataset_state(
        &self,
        source_name: &str,
        dataset_id: &str,
        kind: &str,
    ) -> anyhow::Result<Option<(Option<i64>, Option<String>)>> {
        let rec: Option<(Option<i64>, Option<String>)> = sqlx::query_as(
            r#"
select last_succeeded_import_run_id, last_dataset_version
from bm_meta.dataset_state
where source_name = $1 and dataset_id = $2 and kind = $3
"#,
        )
        .bind(source_name)
        .bind(dataset_id)
        .bind(kind)
        .fetch_optional(&self.pool)
        .await?;
        Ok(rec)
    }

    #[instrument(skip_all, fields(schema = schema, table = table, pk_column = pk_column, pk_value = %pk_value, target_column = target_column))]
    pub async fn get_text_by_pk(
        &self,
        schema: &str,
        table: &str,
        pk_column: &str,
        pk_value: &str,
        target_column: &str,
    ) -> anyhow::Result<Option<String>> {
        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let pk_q = quote_ident(pk_column);
        let target_q = quote_ident(target_column);
        let sql = format!(
            "select {target_q} from {schema_q}.{table_q} where {pk_q} = $1 limit 1"
        );
        let rec: Option<(Option<String>,)> = sqlx::query_as(&sql)
            .bind(pk_value)
            .fetch_optional(&self.pool)
            .await?;
        Ok(rec.and_then(|r| r.0))
    }

    #[instrument(skip_all, fields(import_run_id = import_run_id, table = table_name, pk_column = pk_column, values = values.len()))]
    pub async fn insert_seen_pk_values(
        &self,
        import_run_id: i64,
        table_name: &str,
        pk_column: &str,
        values: &[String],
    ) -> anyhow::Result<()> {
        if values.is_empty() {
            return Ok(());
        }

        let mut qb = sqlx::QueryBuilder::new(
            "insert into src_libgen.seen_pk (import_run_id, table_name, pk_column, pk_value) ",
        );
        qb.push_values(values, |mut b, v| {
            b.push_bind(import_run_id);
            b.push_bind(table_name);
            b.push_bind(pk_column);
            b.push_bind(v);
        });
        qb.push(" on conflict do nothing");
        qb.build().execute(&self.pool).await?;
        Ok(())
    }

    #[instrument(skip_all, fields(schema = schema, table = table, pk_column = pk_column, import_run_id = import_run_id))]
    pub async fn delete_rows_not_seen(
        &self,
        schema: &str,
        table: &str,
        mysql_table_name: &str,
        pk_column: &str,
        import_run_id: i64,
    ) -> anyhow::Result<u64> {
        let schema_q = quote_ident(schema);
        let table_q = quote_ident(table);
        let pk_q = quote_ident(pk_column);

        let sql = format!(
            r#"
delete from {schema_q}.{table_q} t
where not exists (
  select 1
  from src_libgen.seen_pk s
  where s.import_run_id = $1
    and s.table_name = $2
    and s.pk_column = $3
    and s.pk_value = t.{pk_q}
)
"#
        );

        let res = sqlx::query(&sql)
            .bind(import_run_id)
            .bind(mysql_table_name)
            .bind(pk_column)
            .execute(&self.pool)
            .await?;
        Ok(res.rows_affected())
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
