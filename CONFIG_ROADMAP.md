# bulk-merge — TOML Config Roadmap (Control Pane)

This roadmap tracks the configuration surface that must exist in the project’s TOML
control pane.

## Config File

- [ ] Create a canonical config file at `config/bulk-merge.toml`.
- [ ] Define and document config schema in `docs/config.md`.

## Logging & Observability

- [ ] `logging.level` (default, overrides, CLI interaction).
- [ ] `logging.format` (human vs json).
- [ ] `logging.include_target` / `logging.include_location` toggles.

## Execution Policy

- [ ] `execution.dry_run_default` (default false; overridable by CLI).
- [ ] `execution.concurrency` (default and bounds).
- [ ] `execution.retry.max_attempts`.
- [ ] `execution.retry.backoff_ms_initial`.
- [ ] `execution.retry.backoff_ms_max`.
- [ ] `execution.timeout_ms_per_target`.

## Input & Filtering

- [ ] `input.source` (file/stdin/query) with source-specific parameters.
- [ ] `input.allow_duplicates` policy.
- [ ] `filter.include` / `filter.exclude` patterns.

## Output

- [ ] `output.format` (human/json).
- [ ] `output.color` (auto/always/never).
- [ ] `output.report_path` (optional file output).

## Backend Selection

- [ ] `backend.kind` (e.g., `git` or `github`) and per-backend settings tables.

