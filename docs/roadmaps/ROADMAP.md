# bulk-merge — Implementation Roadmap

`bulk-merge` converts large, messy bibliographic metadata dumps into a PostgreSQL
database as usable, queryable tables.

Phase 1 is LibGen-only ingestion (no cross-source merging). Future phases add
other sources (Open Library, OpenAlex, Crossref, Wikidata) using the same import
framework.

## Project Definition

- [ ] Document project goal/non-goals and Phase 1 scope in `README.md`.

## CLI Surface

- [ ] Define CLI surface for LibGen ingestion in `docs/cli.md` (commands, args, exit codes).

## Crate Structure

- [ ] Establish crate layout (`lib.rs` + `main.rs`) and module boundaries for adapters/backends.

## Logging & Error Handling

- [ ] Add structured logging via `tracing` + `tracing-subscriber` (configurable via TOML and CLI).
- [ ] Add error strategy (`thiserror` for domain, `anyhow` at CLI boundaries with context).

## Configuration (Control Pane)

- [ ] Add TOML config loading + validated config model (the control pane).
- [ ] Add `--config` flag and env override support (documented).

## CLI Implementation

- [ ] Add `clap`-based CLI parsing with help/version output.
- [ ] Add `--dry-run` support for all mutating commands.

## Database Foundations

- [ ] Add Postgres connection + migrations for `bm_meta` import bookkeeping.

## Testing

- [ ] Add baseline unit tests for config parsing/validation and CLI argument parsing.

## LibGen Ingestion (Phase 1)

- [ ] Implement LibGen SQL dump parser (MySQL dump subset: `CREATE TABLE`, `INSERT ... VALUES`, escaping, NULL/numbers).
- [ ] Implement offline conversion path: dump → intermediate (TSV/CSV/JSONL) → Postgres `COPY`.
- [ ] Implement streaming ingestion path: dump → batched Postgres load (client-side `COPY` preferred).
- [ ] Implement per-dump-type schema isolation: dedicated tables for `fiction` and `compact` dumps.
- [ ] Map MySQL fields 1-to-1 into Postgres columns (no semantic normalization in Phase 1).
- [ ] Create indexes only after bulk loads finish (post-load indexing) to maximize ingest speed.
- [ ] Implement resumable import with checkpoints and per-file accounting in `bm_meta`.
- [ ] Implement `ingest libgen` command that accepts either dump type and provisions tables automatically.
- [ ] Implement `update libgen` (incremental) command to ingest a newer dump and apply changes incrementally.
- [ ] Implement verification/stats commands for LibGen ingestion (`stats`, `sample`, `validate`).

## Additional Sources (Future)

- [ ] Add Open Library adapter crate and schema (`src_openlibrary`).
- [ ] Add OpenAlex adapter crate and schema (`src_openalex`).
- [ ] Add Crossref adapter crate and schema (`src_crossref`).
- [ ] Add Wikidata adapter crate and schema (`src_wikidata`).
