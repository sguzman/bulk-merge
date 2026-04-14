# bulk-merge — TOML Config Roadmap (Control Pane)

This roadmap tracks the TOML configuration surface that acts as the project’s
control pane. Any tuning knob, limit, or operational policy that is likely to
vary across environments should be expressed here rather than hardcoded.

## Canonical Config

- [ ] Create a canonical config file at `config/bulk-merge.toml`.
- [ ] Define and document config schema in `docs/config.md` (tables + examples).

## Logging & Observability

- [ ] `logging.level` (default and per-module overrides).
- [ ] `logging.format` (`human` | `json`).
- [ ] `logging.include_target` / `logging.include_location` toggles.

## PostgreSQL Target (Connection + Namespacing)

- [ ] `postgres.url` (connection string including credentials/host/db).
- [ ] `postgres.statement_timeout_ms`.
- [ ] `postgres.schema_meta` (default `bm_meta`).
- [ ] `postgres.schema_libgen` (default `src_libgen`).
- [ ] `postgres.table_prefix` (optional naming policy for provisioned tables).

## PostgreSQL Pooling & Performance

- [ ] `postgres.pool.max_connections`.
- [ ] `postgres.pool.min_connections`.
- [ ] `postgres.pool.acquire_timeout_ms`.

## Import Execution Policy

- [ ] `execution.dry_run_default` (default false; overridable by CLI).
- [ ] `execution.concurrency` (bounds and defaults for parsing/loading).
- [ ] `execution.batch.max_rows` (for COPY/flush boundaries).
- [ ] `execution.batch.max_bytes` (memory guardrail).
- [ ] `execution.retry.max_attempts`.
- [ ] `execution.retry.backoff_ms_initial`.
- [ ] `execution.retry.backoff_ms_max`.

## LibGen Input & Parsing Policy

- [ ] `libgen.dump.kind` (`fiction` | `compact`).
- [ ] `libgen.dump.path` (file path or directory policy).
- [ ] `libgen.dump.dataset_id` (stable identifier for checkpointing/incrementals).
- [ ] `libgen.dump.allow_invalid_utf8` policy.
- [ ] `libgen.dump.max_statement_bytes` (guardrail).

## LibGen Table Provisioning

- [ ] `libgen.tables.fiction.name` (or naming template).
- [ ] `libgen.tables.compact.name` (or naming template).
- [ ] `libgen.tables.raw.name` (raw landing table name/template).

## LibGen Resumability & Checkpointing

- [ ] `libgen.resume.enabled` toggle.
- [ ] `libgen.resume.checkpoint_granularity` (statement/row/file offset policy).

## LibGen Incremental Updates

- [ ] `libgen.incremental.strategy` (primary-key vs row-hash).
- [ ] `libgen.incremental.primary_key_columns` (per dump kind).
- [ ] `libgen.incremental.row_hash.enabled` and algorithm name.
- [ ] `libgen.incremental.apply_deletes` (tombstones vs keep-old).

## Output & Reporting

- [ ] `output.format` (`human` | `json`).
- [ ] `output.color` (`auto` | `always` | `never`).
- [ ] `output.report_path` (optional file output).

