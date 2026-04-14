# bulk-merge — Implementation Roadmap

`bulk-merge` converts large, messy bibliographic metadata dumps into a PostgreSQL
database as usable, queryable tables.

Phase 1 is LibGen-only ingestion (no cross-source merging). Future phases add
other sources (Open Library, OpenAlex, Crossref, Wikidata) using the same import
framework.

## Project Definition

- [x] Document project goal/non-goals and Phase 1 scope in `README.md`.

## CLI Surface

- [x] Define CLI surface for LibGen ingestion in `docs/cli.md` (commands, args, exit codes).

## Crate Structure

- [x] Establish crate layout (`lib.rs` + `main.rs`) and module boundaries for adapters/backends.

## Logging & Error Handling

- [x] Add structured logging via `tracing` + `tracing-subscriber` (configurable via TOML and CLI).
- [x] Add periodic progress logging for long-running operations (configurable interval).
- [x] Add `anyhow` for CLI/top-level error context.
- [ ] Add `thiserror` domain error types (as LibGen parser/ingestion modules are implemented).

## Configuration (Control Pane)

- [x] Add TOML config loading (the control pane).
- [x] Add config validation (fail-fast on invalid/missing settings with clear errors).
- [x] Add `--config` flag (documented in `docs/cli.md`).
- [x] Add basic env override support (documented).

## CLI Implementation

- [x] Add `clap`-based CLI parsing with help/version output.
- [x] Add `--dry-run` support for mutating commands (Phase 1 scaffolding).

## Database Foundations

- [x] Add Postgres connection + migrations for `bm_meta` import bookkeeping.

## Testing

- [x] Add baseline unit tests for config parsing/validation and CLI argument parsing.

## LibGen Ingestion (Phase 1)

- [x] Implement LibGen SQL dump parser (MySQL dump subset: `CREATE TABLE`, `INSERT ... VALUES`, escaping, NULL/numbers).
- [x] Implement offline conversion path: dump → intermediate (TSV) → Postgres `COPY`.
- [x] Implement streaming ingestion path: dump → batched Postgres load (ingest via client-side COPY when enabled).
- [x] Implement per-dump-type isolation: dedicated tables for `fiction` and `compact` dumps (separate table prefixes under one schema).
- [x] Map MySQL fields 1-to-1 into Postgres columns (Phase 1: store as `text`).
- [x] Create indexes only after bulk loads finish (post-load indexing) to maximize ingest speed.
- [x] Implement resumable import with checkpoints (byte-offset) in `bm_meta.import_checkpoint`.
- [x] Implement `ingest libgen` command that accepts either dump type and provisions tables automatically.
- [x] Implement `update libgen` (incremental) command to ingest a newer dump and apply changes incrementally (Phase 1: upsert by configured primary key columns).
- [x] Persist dataset-level state for incremental updates (`bm_meta.dataset_state`).
- [x] Implement verification/stats commands for LibGen ingestion (`stats`, `sample`, `validate`).
- [ ] Implement resumable offline load strategy (restart-safe without manual cleanup).

## Additional Sources (Future)

- [ ] Add Open Library adapter crate and schema (`src_openlibrary`).
- [ ] Add OpenAlex adapter crate and schema (`src_openalex`).
- [ ] Add Crossref adapter crate and schema (`src_crossref`).
- [ ] Add Wikidata adapter crate and schema (`src_wikidata`).
