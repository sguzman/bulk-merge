# LibGen Conversion → PostgreSQL Roadmap (Phase 1)

End goal: point `bulk-merge` at a LibGen `fiction` dump or `libgen compact` dump
and ingest it into PostgreSQL into dedicated tables for that dump type, with
incremental updates when pointed at a newer dump.

Notes:
- This roadmap is strictly LibGen ingestion (no cross-source merge).
- Each actionable item is implementable and verifiable in this repo (build/tests).

## Tranche 1 — CLI + Config + Meta Bookkeeping (Prereqs)

- [ ] Add `clap` CLI skeleton with `libgen ingest` and `libgen update` subcommands.
- [ ] Add TOML config model for Postgres + LibGen ingestion settings.
- [ ] Add `tracing` setup with log level/format configurable via TOML and CLI overrides.
- [ ] Add Postgres migrations for `bm_meta` (`import_run`, `import_file`, `import_checkpoint`).
- [ ] Add `src_libgen` schema creation and schema-version tracking.

## Tranche 2 — Dump Detection + Table Provisioning

- [ ] Implement dump kind detection (`fiction` vs `compact`) from CLI flag and/or file heuristics.
- [ ] Define dedicated Postgres table naming strategy per dump kind (configurable prefix).
- [ ] Implement table provisioning for each kind, keeping schemas isolated (no mixing).
- [ ] Store raw rows in a raw landing table with provenance (`import_run_id`, file, line/offset, sha256).

## Tranche 3 — MySQL Dump Parser (Subset)

- [ ] Parse `CREATE TABLE` to capture column order and rough types (ignore engine/charset noise).
- [ ] Parse `INSERT INTO ... VALUES (...)` including multi-row inserts.
- [ ] Correctly decode MySQL string escapes, NULL, numbers, and backtick identifiers.
- [ ] Add parser unit tests with fixtures for tricky escaping and multi-row inserts.

## Tranche 4 — Offline Conversion Path (File Artifact + COPY)

- [ ] Implement converter: MySQL dump → normalized intermediate (TSV/CSV/JSONL; choose one and document).
- [ ] Implement Postgres loader using `COPY` into raw landing and/or staging tables.
- [ ] Add checkpoints so conversion/loading can resume without reprocessing completed regions.
- [ ] Add tests that run conversion on a small fixture dump and load into a temp Postgres (if feasible in CI).

## Tranche 5 — Streaming Ingestion Path (No Intermediate Files)

- [ ] Implement streaming loader: parse dump and feed batched loads (client-side COPY preferred).
- [ ] Implement bounded buffering (`max_rows`/`max_bytes`) and backpressure.
- [ ] Reuse the same parser core as offline conversion (two sinks).
- [ ] Add resumability for streaming ingestion via `bm_meta` checkpoints (resume from last processed offset/line).

## Tranche 6 — Incremental Updates (Newer Dumps)

- [ ] Define stable per-kind key strategy (config: primary key columns; fallback to row-hash).
- [ ] Implement `libgen update` that imports a newer dump version and applies changes incrementally.
- [ ] Persist update state in `bm_meta` (dataset_id, last ingested version, checkpoints).
- [ ] Implement configurable delete handling (tombstones vs keep-old).
- [ ] Add tests for incremental apply logic using two fixture dumps (v1 → v2).

## Tranche 7 — Operator-Facing Commands

- [ ] Implement `libgen stats` command: counts, newest import run, basic sanity metrics.
- [ ] Implement `libgen sample` command: print N rows (human/json output).
- [ ] Implement `libgen validate` command: minimal invariants (row counts non-zero, key uniqueness if configured).
