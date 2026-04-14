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

Registers an import run for a LibGen dump (Phase 1 ingestion to be implemented).

### `bulk-merge libgen update --kind <fiction|compact> --dump <path>`

Registers an update run for a newer LibGen dump (Phase 1 incrementals to be implemented).

