# CLI

## Global flags

- `--config <path>`: TOML control pane path (default: `config/bulk-merge.toml`)
- `--dry-run`: do not perform mutating operations
- `--log-level <level>`: override log level (e.g. `info`, `debug`, `trace`)
- `--log-format <human|json>`: override log format

## Commands

### `bulk-merge init-db`

Creates base schemas and bookkeeping tables:

- `bm_meta.*` for import runs/checkpoints
- `src_libgen` schema (Phase 1)

### `bulk-merge libgen ingest --kind <fiction|compact> --dump <path>`

Phase 1 current behavior:

- registers an import run in `bm_meta.import_run`
- scans the dump for `CREATE TABLE` statements and provisions dedicated PostgreSQL tables
  under `src_libgen` using the configured per-kind prefixes
- parses `INSERT INTO ... VALUES` and bulk-inserts rows into the provisioned tables (Phase 1: all columns stored as `text`)
- writes resumability checkpoints (byte offset) in `bm_meta.import_checkpoint` when enabled
- creates post-load indexes after ingest when `postgres.indexing.create_after_load = true`

COPY-based loading is not implemented yet.

### `bulk-merge libgen update --kind <fiction|compact> --dump <path>`

Phase 1 current behavior:

- registers an import run in `bm_meta.import_run`
- provisions tables from the dump schema (same as `ingest`)
- ingests rows via upsert (`ON CONFLICT`) using `libgen.incremental.primary_key_columns` (full scan; applies changes incrementally)
- when `libgen.incremental.apply_deletes = true` and the PK is a single column, deletes rows not present in the new dump

### `bulk-merge libgen stats`

Print counts and recent import run metadata:

- number of provisioned tables per kind (by prefix)
- last 5 `bm_meta.import_run` entries for `source_name = 'libgen'`
- raw statement count for each run

### `bulk-merge libgen sample --kind <fiction|compact> --mysql-table <name> [--limit <n>]`

Sample rows from an ingested table.

Notes:

- `--kind` selects the configured table prefix.
- `--mysql-table` is the original MySQL table name (without prefixes).

### `bulk-merge libgen validate --kind <fiction|compact> --mysql-table <name>`

Validate minimal invariants.

Current checks:

- row count of the resolved table is > 0
