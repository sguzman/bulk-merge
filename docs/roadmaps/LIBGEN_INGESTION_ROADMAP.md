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
- [x] `bulk-merge libgen ingest ...` provisions dedicated tables from dump schema (`CREATE TABLE` discovery).
- [x] `bulk-merge libgen ingest ...` ingests row data into the dedicated tables.
- [x] `bulk-merge libgen update ...` registers an `import_run` (Phase 1 scaffolding).
- [x] `bulk-merge libgen update ...` provisions tables from dump schema (`CREATE TABLE` discovery).
- [x] `bulk-merge libgen update ...` applies an incremental row-level update from a newer dump (Phase 1: full scan + upsert by primary key).
- [x] `bulk-merge libgen stats` prints counts and last-run info.
- [x] `bulk-merge libgen sample` prints a small sample of rows (Phase 1: JSON via `row_to_json`).
- [x] `bulk-merge libgen validate` runs minimal invariants and reports failures (Phase 1: row count > 0).

## Configuration & Policies (Control Pane)

- [x] TOML config includes Postgres connection details (credentials/host/db) and core tunables (pooling, batching, retries).
- [ ] TOML config includes full schema/table naming policy for LibGen provisioned tables.
  - [x] Configurable `paths.cache_dir` base for all intermediate artifacts and temp outputs.
  - [ ] Configurable offline artifact layout under cache dir (per-kind subdir naming policy).
  - [ ] Configurable dataset naming policy for offline load (how `dataset_id` is chosen when absent).
- [x] TOML config includes LibGen dump kind and resumability/incremental strategy knobs (initial surface).
- [x] TOML config includes LibGen dump path and dataset_id as first-class settings.
- [x] CLI can override high-value runtime knobs (log level/format, config path, dry-run).

## Import Bookkeeping (`bm_meta`)

- [x] Migrations for `bm_meta.import_run`, `bm_meta.import_file`, `bm_meta.import_checkpoint`.
- [x] Every ingest/update creates an `import_run` row with config snapshot (Phase 1 scaffolding).
- [x] Per-file accounting tracks progress (bytes/records/offsets) and supports resume.

## Table Provisioning (Dedicated Tables per Kind)

- [x] Dedicated table naming strategy for `fiction` vs `compact` (configurable prefixes; never mixed).
- [x] Provision kind-specific table(s) on demand from `CREATE TABLE` schema discovery.
- [x] Provision raw landing table(s) for provenance-preserving reprocessing (`src_libgen.raw_statement`).
- [x] Store provenance for raw statements (`import_run_id`, byte offset, sha256, kind, mysql_table).
- [x] Map LibGen table columns 1-to-1 from the MySQL dump into PostgreSQL columns (Phase 1: store as `text` for ingest robustness).

## MySQL Dump Parser (Supported Subset)

- [x] Parse `CREATE TABLE` to capture column order and rough types (ignore engine/charset noise).
- [x] Parse `INSERT INTO ... VALUES (...)` including multi-row inserts.
- [x] Correctly decode MySQL string escapes, NULL, numbers, and backtick identifiers.
- [ ] Guardrails: maximum statement size, bounded buffering, explicit error reporting with context.
  - [ ] Error contexts include dump offset, statement kind guess, and a short statement preview (bounded) for debugging.
  - [ ] Explicitly cap per-statement memory allocation and propagate a typed “statement too large” error.
  - [ ] Add parser tests that assert guardrail errors are thrown with context (no panics).
- [x] Parser unit tests with fixtures (CREATE TABLE + INSERT parsing basics).

## Offline Conversion Path (Intermediate Artifact + COPY)

- [x] Convert MySQL dump → normalized intermediate format (TSV; documented choice).
- [x] Load intermediate into Postgres using `COPY` (fast path).
- [x] Resumability: checkpoints allow restarting without reprocessing completed regions.
- [x] Create indexes only after bulk insert finishes (post-load indexing) to maximize ingest speed.
- [ ] Offline load resumability: restart-safe loads without manual cleanup (choose and implement one strategy).
  - [ ] Strategy A: staging tables + atomic swap/rename (requires a swap plan + index rebuild policy).
  - [ ] Strategy B: run-scoped truncate+reload (requires strong run identity + explicit “unsafe” knob).
  - [ ] Strategy C: per-table checkpoints + dedupe/upsert (requires stable keys for each table or row-hash).
  - [ ] Add an integration test that simulates an interrupted offline load and verifies restart behavior.
- [ ] Cache policy: all on-disk intermediate artifacts and temp outputs default under `./.cache/bulk-merge/` (configurable root).
  - [x] Default offline artifacts under `paths.cache_dir` when no explicit output dir is provided (via derived `libgen.offline.out_dir_default`).
  - [x] `bulk-merge libgen convert` supports explicit `--out-dir` override (bypasses cache policy).
  - [ ] `bulk-merge libgen convert` defaults to writing into a kind-specific cache dir rooted at `paths.cache_dir` (e.g. `${paths.cache_dir}/libgen-offline/{fiction|compact}`).
  - [ ] Document cache directory contents and cleanup expectations (no manual QA; just doc).

## Streaming Ingestion Path (No Intermediate Files)

- [x] Parse dump and feed batched loads directly to Postgres (ingest uses client-side COPY when enabled; update uses upsert).
- [x] Resumability: checkpoints allow resuming from last processed offset/line.
- [x] Backpressure and bounded memory (`max_rows`/`max_bytes` + `execution.memory_hard_limit_bytes`).
- [x] Create indexes only after streaming ingest finishes (post-load indexing) to maximize ingest speed.

## Incremental Updates (Newer Dumps)

- [x] Define stable per-kind key strategy (config: primary key columns; optional row-hash de-dupe).
- [x] `libgen update` imports a newer dump and applies changes incrementally (Phase 1: full scan + upsert by primary key).
- [x] Persist incremental update state in `bm_meta` (`dataset_id`, last ingested version).
- [x] Configurable delete handling (tombstones vs keep-old) (Phase 1: delete rows not seen in new dump when enabled and PK is single-column).
- [x] Tests for incremental apply logic using two fixture dumps (v1 → v2).
