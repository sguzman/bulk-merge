# LibGen Conversion → PostgreSQL Roadmap (Phase 1)

End goal: point `bulk-merge` at a LibGen `fiction` dump or `libgen compact` dump
and ingest it into PostgreSQL into dedicated tables for that dump kind. Pointing
at a newer dump should update the corresponding table incrementally.

Scope notes:
- This roadmap is strictly LibGen ingestion (no cross-source merge).
- Items are organized by feature area (not by work tranche).

## CLI Commands (Operator Surface)

- [x] `bulk-merge init-db` provisions `bm_meta` and the `src_libgen` schema.
- [ ] `bulk-merge init-db` provisions LibGen kind-specific tables (once schema discovery exists).
- [x] `bulk-merge libgen ingest ...` registers an `import_run` (Phase 1 scaffolding).
- [ ] `bulk-merge libgen ingest ...` ingests a dump into the dedicated tables.
- [x] `bulk-merge libgen update ...` registers an `import_run` (Phase 1 scaffolding).
- [ ] `bulk-merge libgen update ...` applies an incremental update from a newer dump.
- [x] `bulk-merge libgen stats` command exists (placeholder).
- [ ] `bulk-merge libgen stats` prints real counts and last-run info.
- [x] `bulk-merge libgen sample` command exists (placeholder).
- [ ] `bulk-merge libgen sample` prints a small sample of rows (human/json).
- [x] `bulk-merge libgen validate` command exists (placeholder).
- [ ] `bulk-merge libgen validate` runs minimal invariants and reports failures.

## Configuration & Policies (Control Pane)

- [x] TOML config includes Postgres connection details (credentials/host/db) and core tunables (pooling, batching, retries).
- [ ] TOML config includes full schema/table naming policy for LibGen provisioned tables.
- [x] TOML config includes LibGen dump kind and resumability/incremental strategy knobs (initial surface).
- [ ] TOML config includes LibGen dump path and dataset_id as first-class settings.
- [x] CLI can override high-value runtime knobs (log level/format, config path, dry-run).

## Import Bookkeeping (`bm_meta`)

- [x] Migrations for `bm_meta.import_run`, `bm_meta.import_file`, `bm_meta.import_checkpoint`.
- [x] Every ingest/update creates an `import_run` row with config snapshot (Phase 1 scaffolding).
- [ ] Per-file accounting tracks progress (bytes/records/offsets) and supports resume.

## Table Provisioning (Dedicated Tables per Kind)

- [ ] Dedicated table naming strategy for `fiction` vs `compact` (configurable; never mixed).
- [ ] Provision raw landing table(s) and the kind-specific table(s) on demand.
- [ ] Store provenance on raw rows (`import_run_id`, file, line/offset, sha256).
- [ ] Map LibGen table columns 1-to-1 from the MySQL dump into PostgreSQL columns (no semantic normalization yet).

## MySQL Dump Parser (Supported Subset)

- [ ] Parse `CREATE TABLE` to capture column order and rough types (ignore engine/charset noise).
- [ ] Parse `INSERT INTO ... VALUES (...)` including multi-row inserts.
- [ ] Correctly decode MySQL string escapes, NULL, numbers, and backtick identifiers.
- [ ] Guardrails: maximum statement size, bounded buffering, explicit error reporting with context.
- [ ] Parser unit tests with fixtures (escaping, multi-row inserts, odd quoting).

## Offline Conversion Path (Intermediate Artifact + COPY)

- [ ] Convert MySQL dump → normalized intermediate format (choose one: TSV/CSV/JSONL; document choice).
- [ ] Load intermediate into Postgres using `COPY` (fast path).
- [ ] Resumability: checkpoints allow restarting without reprocessing completed regions.
- [ ] Create indexes only after bulk insert finishes (post-load indexing) to maximize ingest speed.

## Streaming Ingestion Path (No Intermediate Files)

- [ ] Parse dump and feed batched loads directly to Postgres (client-side COPY preferred).
- [ ] Resumability: checkpoints allow resuming from last processed offset/line.
- [ ] Backpressure and bounded memory (`max_rows`/`max_bytes`).
- [ ] Create indexes only after streaming ingest finishes (post-load indexing) to maximize ingest speed.

## Incremental Updates (Newer Dumps)

- [ ] Define stable per-kind key strategy (config: primary key columns; fallback to row-hash).
- [ ] `libgen update` imports a newer dump and applies changes incrementally.
- [ ] Persist incremental update state in `bm_meta` (`dataset_id`, last ingested version, checkpoints).
- [ ] Configurable delete handling (tombstones vs keep-old).
- [ ] Tests for incremental apply logic using two fixture dumps (v1 → v2).
