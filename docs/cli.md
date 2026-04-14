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
- ingests rows the same way as `ingest` (incremental update logic is not implemented yet)

Incremental row-level updates are not implemented yet.
