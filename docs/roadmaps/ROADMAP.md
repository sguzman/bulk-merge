# bulk-merge — Implementation Roadmap

`bulk-merge` converts large, messy bibliographic metadata dumps into a PostgreSQL
database as usable, queryable tables.

Phase 1 is LibGen-only ingestion (no cross-source merging). Future phases add
other sources (Open Library, OpenAlex, Crossref, Wikidata) using the same import
framework.

## Tranche 0 — LibGen-First Ingestion Foundations

- [ ] Document project goal/non-goals and Phase 1 scope in `README.md`.
- [ ] Define CLI surface for LibGen ingestion in `docs/cli.md` (commands, args, exit codes).
- [ ] Establish crate layout (`lib.rs` + `main.rs`) and module boundaries for adapters/backends.
- [ ] Add structured logging via `tracing` + `tracing-subscriber` (configurable via TOML and CLI).
- [ ] Add error strategy (`thiserror` for domain, `anyhow` at CLI boundaries with context).
- [ ] Add TOML config loading + validated config model (the control pane).
- [ ] Add `--config` flag and env override support (documented).
- [ ] Add `clap`-based CLI parsing with help/version output.
- [ ] Add `--dry-run` support for all mutating commands.
- [ ] Add Postgres connection + migrations for `bm_meta` import bookkeeping.
- [ ] Add baseline unit tests for config parsing/validation and CLI argument parsing.

## Tranche 1 — LibGen Dump Conversion (Proof of Concept)

- [ ] Implement LibGen SQL dump parser (MySQL dump subset: `CREATE TABLE`, `INSERT ... VALUES`, escaping, NULL/numbers).
- [ ] Implement offline conversion path: dump → intermediate (TSV/CSV/JSONL) → Postgres `COPY`.
- [ ] Implement streaming ingestion path: dump → batched Postgres load (client-side `COPY` preferred).
- [ ] Implement per-dump-type schema isolation: dedicated tables for `fiction` and `compact` dumps.
- [ ] Implement resumable import with checkpoints and per-file accounting in `bm_meta`.
- [ ] Implement `ingest libgen` command that accepts either dump type and provisions tables automatically.
- [ ] Implement `update libgen` (incremental) command to ingest a newer dump and apply changes incrementally.
- [ ] Implement verification/stats commands for LibGen ingestion (`stats`, `sample`, `validate`).

## Tranche 2 — Additional Sources (Not in Phase 1)

- [ ] Add Open Library adapter crate and schema (`src_openlibrary`).
- [ ] Add OpenAlex adapter crate and schema (`src_openalex`).
- [ ] Add Crossref adapter crate and schema (`src_crossref`).
- [ ] Add Wikidata adapter crate and schema (`src_wikidata`).

