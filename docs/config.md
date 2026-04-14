# Configuration (TOML control pane)

All operational settings live in the TOML control pane (default: `config/bulk-merge.toml`).
This includes PostgreSQL target information (credentials/host/db), schema/table naming
policies, and ingest tuning knobs (batch sizes, concurrency, retries, etc.).

## PostgreSQL

Minimum required:

- `postgres.url`: `postgresql://user:password@host:port/dbname`

Environment overrides:

- `BULK_MERGE_POSTGRES_URL`

Schemas used by the project:

- `postgres.schema_meta` (default `bm_meta`)
- `postgres.schema_libgen` (default `src_libgen`)

## Logging

- `logging.level`: `trace|debug|info|warn|error`
- `logging.format`: `human|json`

Environment overrides:

- `BULK_MERGE_LOG_LEVEL`
- `BULK_MERGE_LOG_FORMAT` (`human|json`)
