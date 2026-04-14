# bulk-merge — TOML Config Roadmap (Control Pane)

This roadmap tracks the TOML configuration surface that acts as the project’s
control pane. Any tuning knob or operational policy that is likely to vary across
environments should be expressed here rather than hardcoded.

## Canonical Config

- [ ] Create a canonical config file at `config/bulk-merge.toml`.
- [ ] Define and document config schema in `docs/config.md` (tables + examples).

## Logging & Observability

- [ ] `logging.level` (default and per-module overrides).
- [ ] `logging.format` (`human` | `json`).
- [ ] `logging.include_target` / `logging.include_location` toggles.

## Postgres

- [ ] `postgres.url` (connection string) and `postgres.statement_timeout_ms`.
- [ ] `postgres.schema_meta` (default `bm_meta`) and `postgres.schema_libgen` (default `src_libgen`).
- [ ] `postgres.migrations.table` (if custom tracking is needed).

## Import Execution Policy

- [ ] `execution.dry_run_default` (default false; overridable by CLI).
- [ ] `execution.concurrency` (bounds and defaults for parsing/loading).
- [ ] `execution.batch.max_rows` (for COPY/flush boundaries).
- [ ] `execution.batch.max_bytes` (memory guardrail).
- [ ] `execution.retry.max_attempts`.
- [ ] `execution.retry.backoff_ms_initial`.
- [ ] `execution.retry.backoff_ms_max`.

## LibGen Input

- [ ] `libgen.dump.kind` (`fiction` | `compact`).
- [ ] `libgen.dump.path` (file path or directory path policy).
- [ ] `libgen.dump.encoding` (if needed) and `libgen.dump.allow_invalid_utf8` policy.
- [ ] `libgen.dump.table_prefix` (for dedicated table naming).
- [ ] `libgen.dump.dataset_id` (stable identifier for incrementals/checkpoints).

## LibGen Incremental Update Policy

- [ ] `libgen.incremental.strategy` (keyed by stable primary key vs row-hash).
- [ ] `libgen.incremental.primary_key_columns` (per dump kind).
- [ ] `libgen.incremental.row_hash` (enabled/disabled and algorithm name).
- [ ] `libgen.incremental.apply_deletes` (tombstone handling).

## Output & Reporting

- [ ] `output.format` (`human` | `json`).
- [ ] `output.color` (`auto` | `always` | `never`).
- [ ] `output.report_path` (optional file output).

