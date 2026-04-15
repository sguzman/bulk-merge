# bulk-merge — TOML Config Roadmap (Control Pane)

This roadmap tracks the TOML configuration surface that acts as the project’s
control pane. Any tuning knob, limit, or operational policy that is likely to
vary across environments should be expressed here rather than hardcoded.

## Canonical Config

- [x] Create a canonical config file at `config/bulk-merge.toml`.
- [ ] Define and document full config schema in `docs/config.md` (tables + examples).

## Paths & Cache Policy

- [x] `paths.cache_dir` (default `./.cache/bulk-merge`; base for intermediate artifacts and temp outputs).
- [x] `paths.cache_policy` (`always|prefer|never`) controlling whether commands default into cache dir or require explicit paths.
- [x] `libgen.offline.out_dir_default` derived from `paths.cache_dir` by default (unless explicitly set).

## Logging & Observability

- [x] `logging.level` (default and per-module overrides).
- [x] `logging.format` (`human` | `json`).
- [x] `logging.include_target` / `logging.include_location` toggles.

## PostgreSQL Target (Connection + Namespacing)

- [x] `postgres.url` optional override (connection string including credentials/host/db).
- [x] `postgres.host` / `postgres.port` / `postgres.user` / `postgres.password` / `postgres.database`.
- [x] `postgres.statement_timeout_ms`.
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
- [x] `execution.loader.kind` (`copy`|`insert`).
- [x] `execution.copy.file_send_chunk_bytes` (chunk size when streaming intermediate files into `COPY FROM STDIN`).
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
- [x] `libgen.dump.error_preview_bytes` (bounded statement preview for parse errors).
- [x] `libgen.dump.sanitize_nul_bytes` and `libgen.dump.nul_replacement` (Postgres text compatibility).

## LibGen typing policy

- [x] `libgen.typing.mode` (best-effort typed columns vs all-text).
- [x] `libgen.typing.unrepresentable_policy` (what to do when a value can't be coerced to the target type).

## LibGen init-db provisioning

- [x] `libgen.init.provision_tables` and `libgen.init.dumps` (optional schema-only provisioning during `init-db`).
- [x] Normalize empty `libgen.init.dumps.*` as unset and validate provision_tables requires at least one dump path.

## LibGen Table Provisioning

- [x] `libgen.tables.fiction.name` (or naming template).
- [x] `libgen.tables.compact.name` (or naming template).
- [x] `libgen.offline.out_dir_default` (default output directory for offline conversion artifacts).
- [x] `libgen.offline.layout` (`kind_subdir|flat`) (offline artifact layout policy under cache dir).
- [x] `libgen.offline.load.*` (offline load strategy + staging/swap policy knobs).
  - [x] `libgen.offline.load.drop_staging_tables_on_success`.
  - [x] `libgen.offline.load.staging_table_suffix_template`.
  - [x] `libgen.offline.load.dataset_id_template`.
  - [x] `libgen.offline.load.resume_strict_manifest_match` (resume safety policy).
- [x] Document raw landing table name is fixed to `src_libgen.raw_statement` (migration-defined).
- [ ] Make raw landing table name configurable (requires migration redesign / dynamic DDL strategy).

## LibGen Resumability & Checkpointing

- [x] `libgen.resume.enabled` toggle.
- [x] `libgen.resume.checkpoint_granularity` (statement/row/file offset policy).

## LibGen Incremental Updates

- [x] `libgen.incremental.strategy` (primary-key vs row-hash).
- [x] `libgen.incremental.primary_key_columns` (per dump kind).
- [x] `libgen.incremental.row_hash.enabled` and algorithm name.
- [x] `libgen.incremental.apply_deletes` (tombstones vs keep-old).

## LibGen Raw Landing

- [x] `libgen.raw.enabled` (store raw statements for replay).
- [x] `libgen.raw.store_other_statements` (store non-INSERT/non-CREATE statements).

## Output & Reporting

- [x] `output.format` (`human` | `json`).
- [x] `output.color` (`auto` | `always` | `never`).
- [x] `output.report_path` (optional file output).
