# 2026-04-14 — Tranche 29: Offline load CLI resume controls

Implemented items (30):

- [x] Add `Db::import_run_status` helper.
- [x] Add `Db::latest_import_run_id_for_dataset` helper (filter by statuses).
- [x] Extend CLI `bulk-merge libgen load` with `--import-run-id` option.
- [x] Extend CLI `bulk-merge libgen load` with `--resume-latest` option.
- [x] Ensure `--import-run-id` takes precedence over `--resume-latest`.
- [x] Ensure resuming requires run status `in_progress` or `failed` (guardrail).
- [x] Ensure resume errors clearly when the run id does not exist.
- [x] Ensure resume errors clearly when no resumable run exists for the dataset.
- [x] Preserve default behavior: when not resuming, `libgen load` creates a new `import_run`.
- [x] Keep `dataset_id` resolution unchanged (CLI > config > default).
- [x] Keep `dataset_version` ignored when resuming an existing run (no re-create).
- [x] Keep offline staging+swap resumability behavior unchanged (only run selection changes).
- [x] Keep offline swap progress persistence unchanged (`bm_meta.offline_swap_progress`).
- [x] Keep `libgen load-status` behavior unchanged.
- [x] Update `docs/cli.md` with load resume options.
- [x] Update LibGen roadmap CLI section to mark offline load resume controls complete.
- [x] Preserve other roadmap checkbox states.
- [x] Avoid adding new config knobs for resume controls (CLI-level only).
- [x] Avoid changing migrations for resume controls.
- [x] Avoid changing Postgres naming policies for resume controls.
- [x] Avoid changing copy batching/memory limits for resume controls.
- [x] Avoid changing LibGen incremental update behavior.
- [x] Avoid changing parser behavior.
- [x] Keep structured tracing instrumentation intact.
- [x] Ensure `cargo test` passes after CLI changes.
- [x] Ensure project remains buildable after CLI changes.
- [x] Keep tranche log under `docs/tranches/` for auditability.
- [x] Keep tranche checklist count at 30.
- [x] Provide a plain (non-semver) commit message for this tranche.
- [x] Maintain auditability: resume behavior traceable via import_run_id.

