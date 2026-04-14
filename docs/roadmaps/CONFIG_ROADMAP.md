# bulk-merge — TOML Config Roadmap (Control Pane)

This roadmap tracks the TOML configuration surface that acts as the project’s
control pane. Any tuning knob, limit, or operational policy that is likely to
vary across environments should be expressed here rather than hardcoded.

## Canonical Config

- [x] Create a canonical config file at `config/bulk-merge.toml`.
- [ ] Define and document full config schema in `docs/config.md` (tables + examples).

## Logging & Observability

- [x] `logging.level` (default and per-module overrides).
- [x] `logging.format` (`human` | `json`).
- [x] `logging.include_target` / `logging.include_location` toggles.

## PostgreSQL Target (Connection + Namespacing)

- [x] `postgres.url` optional override (connection string including credentials/host/db).
- [x] `postgres.host` / `postgres.port` / `postgres.user` / `postgres.password` / `postgres.database`.
- [ ] `postgres.statement_timeout_ms`.
- [x] `postgres.schema_meta` (default `bm_meta`).
- [x] `postgres.schema_libgen` (default `src_libgen`).
- [x] `postgres.table_prefix` (optional naming policy for provisioned tables).

## PostgreSQL Pooling & Performance

- [x] `postgres.pool.max_connections`.
- [x] `postgres.pool.min_connections`.
- [x] `postgres.pool.acquire_timeout_ms`.

## PostgreSQL Indexing Policy (Ingest Speed First)

- [x] `postgres.indexing.create_after_load` (bool; default true).
- [x] `postgres.indexing.concurrent` (bool; optional, for post-load index creation).
- [x] `postgres.indexing.main_fields` (per dump kind list of columns to index for quick search).

## Import Execution Policy

- [x] `execution.dry_run_default` (default false; overridable by CLI).
- [x] `execution.concurrency` (bounds and defaults for parsing/loading).
- [x] `execution.memory_hard_limit_bytes` (global memory guardrail).
- [x] `execution.batch.max_rows` (for COPY/flush boundaries).
- [x] `execution.batch.max_bytes` (memory guardrail).
- [x] `execution.retry.max_attempts`.
- [x] `execution.retry.backoff_ms_initial`.
- [x] `execution.retry.backoff_ms_max`.

## Progress Logging

- [x] `progress.log_interval_seconds` (periodic progress logs for long-running operations).

## LibGen Input & Parsing Policy

- [x] `libgen.dump.kind` (`fiction` | `compact`).
- [x] `libgen.dump.path` (file path or directory policy).
- [x] `libgen.dump.dataset_id` (stable identifier for checkpointing/incrementals).
- [x] `libgen.dump.allow_invalid_utf8` policy.
- [x] `libgen.dump.max_statement_bytes` (guardrail).

## LibGen Table Provisioning

- [x] `libgen.tables.fiction.name` (or naming template).
- [x] `libgen.tables.compact.name` (or naming template).
- [ ] `libgen.tables.raw.name` (raw landing table name/template).

## LibGen Resumability & Checkpointing

- [x] `libgen.resume.enabled` toggle.
- [x] `libgen.resume.checkpoint_granularity` (statement/row/file offset policy).

## LibGen Incremental Updates

- [x] `libgen.incremental.strategy` (primary-key vs row-hash).
- [x] `libgen.incremental.primary_key_columns` (per dump kind).
- [ ] `libgen.incremental.row_hash.enabled` and algorithm name.
- [x] `libgen.incremental.apply_deletes` (tombstones vs keep-old).

## LibGen Raw Landing

- [x] `libgen.raw.enabled` (store raw statements for replay).
- [x] `libgen.raw.store_other_statements` (store non-INSERT/non-CREATE statements).

## Output & Reporting

- [x] `output.format` (`human` | `json`).
- [x] `output.color` (`auto` | `always` | `never`).
- [ ] `output.report_path` (optional file output).
