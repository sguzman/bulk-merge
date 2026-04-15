# Configuration (TOML control pane)

All operational settings live in the TOML control pane (default: `config/bulk-merge.toml`).
This includes PostgreSQL target information (credentials/host/db), schema/table naming
policies, and ingest tuning knobs (batch sizes, concurrency, retries, etc.).

## Paths & cache policy

- `paths.cache_dir`: base directory for cacheable artifacts and temp outputs (default `./.cache/bulk-merge`).
- `paths.cache_policy`: `always|prefer|never` controls whether commands default into `paths.cache_dir` when no explicit output path is provided.

See `docs/cache.md` for the expected on-disk layout and cleanup notes.

## PostgreSQL

Minimum required:

- Either a full URL:
  - `postgres.url`: `postgresql://user:password@host:port/dbname`
- Or discrete connection properties:
  - `postgres.host`
  - `postgres.port`
  - `postgres.user`
  - `postgres.password`
  - `postgres.database`

Environment overrides:

- `BULK_MERGE_POSTGRES_URL`
- `BULK_MERGE_POSTGRES_HOST`
- `BULK_MERGE_POSTGRES_PORT`
- `BULK_MERGE_POSTGRES_USER`
- `BULK_MERGE_POSTGRES_PASSWORD`
- `BULK_MERGE_POSTGRES_DATABASE`

Schemas used by the project:

- `postgres.schema_meta` (default `bm_meta`)
- `postgres.schema_libgen` (default `src_libgen`)

Optional:

- `postgres.statement_timeout_ms`: sets `statement_timeout` after connecting (applies to every pooled connection).

## Logging

- `logging.level`: `trace|debug|info|warn|error`
- `logging.format`: `human|json`

Environment overrides:

- `BULK_MERGE_LOG_LEVEL`
- `BULK_MERGE_LOG_FORMAT` (`human|json`)

## Progress logging (long-running operations)

- `progress.log_interval_seconds`: emits periodic progress logs for long-running operations when total work is known (e.g., file byte offsets).

## Execution limits

- `execution.memory_hard_limit_bytes`: global memory guardrail for long-running ingestion steps.
- `execution.batch.max_rows` / `execution.batch.max_bytes`: bounded buffering thresholds for batch writes.
- `execution.loader.kind`: `copy|insert` (ingest path; update uses upsert).
- `execution.copy.file_send_chunk_bytes`: chunk size used when streaming intermediate TSV files into `COPY FROM STDIN`.

## LibGen offline mode (intermediate TSV)

- `libgen.offline.out_dir_default`: default directory for `bulk-merge libgen convert` output (`manifest.json`, `state.json`, `*.tsv`).
  - If omitted, defaults to `${paths.cache_dir}/libgen-offline` unless `paths.cache_policy = "never"`.
- `libgen.offline.layout`: `kind_subdir|flat` controls whether offline artifacts are written into `${out_dir_default}/{fiction|compact}` or directly into `out_dir_default`.
- `libgen.offline.load.strategy`: currently `staging_swap` (load into a staging schema, then rename into place).
- `libgen.offline.load.staging_schema_prefix`: prefix used to create per-run staging schemas.
- `libgen.offline.load.dataset_id_template`: default dataset id template used when `--dataset-id` and `libgen.dump.dataset_id` are absent (supports `{kind}`).
- `libgen.offline.load.keep_old_tables`: keep the previous live table as `${table}__old_<run_id>` when swapping in staging tables.
- `libgen.offline.load.drop_old_tables_on_success`: drop any kept old tables at the end of a successful load (default false).
- `libgen.offline.load.drop_staging_schema_on_success`: drop the per-run staging schema after a successful load (default false).
- `libgen.offline.load.resume_strict_manifest_match`: when resuming an existing import run, require the manifest kind/dump path to match the recorded import_run config (default true).

## LibGen init-db provisioning (optional)

- `libgen.init.provision_tables`: when true, `bulk-merge init-db` will scan configured dump paths for `CREATE TABLE` statements and provision the corresponding Postgres tables (no row ingest).
- `libgen.init.dumps.fiction`: dump path used for schema discovery for fiction tables (optional).
- `libgen.init.dumps.compact`: dump path used for schema discovery for compact tables (optional).

Notes:
- When `libgen.init.provision_tables = true`, at least one of the dump paths must be set.
- Empty strings are treated as unset.

## LibGen incremental strategy

- `libgen.incremental.strategy`: `primary_key|row_hash`
- `libgen.incremental.primary_key_columns`: per-kind PK columns used for `ON CONFLICT` updates
- `libgen.incremental.row_hash.enabled`: when enabled with `strategy="row_hash"`, adds `_bm_row_hash` to provisioned tables and de-dupes by row hash

Note: `row_hash` provides idempotent ingestion/de-duplication, but it does not model “updates” unless the source provides stable keys.

## LibGen raw landing (provenance)

- Raw statements are stored in `src_libgen.raw_statement` when `libgen.raw.enabled = true`.
- This table name is currently migration-defined (not yet configurable via TOML).

## Output reporting

- `output.report_path`: when set, appends JSON lines describing command outputs (useful for automation/log shipping).

## LibGen parsing guardrails

- `libgen.dump.max_statement_bytes`: hard cap for in-memory statement buffering.
- `libgen.dump.error_preview_bytes`: max bytes of a statement preview included in parse errors for debugging.
