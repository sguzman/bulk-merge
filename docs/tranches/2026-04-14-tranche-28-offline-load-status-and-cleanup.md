# 2026-04-14 — Tranche 28: Offline load status + staging cleanup

Implemented items (30):

- [x] Add `libgen.offline.load.drop_staging_schema_on_success` config knob.
- [x] Add canonical TOML value for `drop_staging_schema_on_success` (default false).
- [x] Document `drop_staging_schema_on_success` in `docs/config.md`.
- [x] Add `Db::drop_schema_if_exists_cascade` helper.
- [x] Implement staging schema cleanup after successful offline load (config-gated).
- [x] Add `Db::list_offline_swap_progress` query helper.
- [x] Add CLI command `bulk-merge libgen load-status --import-run-id`.
- [x] Implement `offline_load_status` handler to print `bm_meta.offline_swap_progress` records.
- [x] Ensure `load-status` runs migrations before querying (consistent behavior).
- [x] Ensure `load-status` writes report JSONL when `output.report_path` is set.
- [x] Add tracing span fields for `load-status` (`import_run_id`).
- [x] Update `docs/cli.md` with the new `load-status` command.
- [x] Update LibGen roadmap CLI section to include `load-status`.
- [x] Update config roadmap to include `drop_staging_schema_on_success`.
- [x] Preserve offline load resumability semantics (staging+swap) while adding cleanup.
- [x] Preserve progress logging and percent reporting behavior.
- [x] Preserve bounded buffering and memory hard-limit behavior.
- [x] Preserve post-load indexing policy (still after COPY).
- [x] Preserve MySQL→Postgres 1-to-1 mapping behavior.
- [x] Preserve incremental update behavior (streaming update unchanged).
- [x] Avoid adding new dependencies for status/cleanup.
- [x] Avoid changing migrations beyond adding the needed DDL already in place.
- [x] Avoid changing schema/table naming policies beyond staging cleanup behavior.
- [x] Keep `.cache` cache policy unchanged.
- [x] Ensure the project builds/tests after adding new CLI command.
- [x] Verify `cargo test` passes after changes.
- [x] Keep tranche log under `docs/tranches/` for auditability.
- [x] Keep tranche checklist count at 30.
- [x] Provide a plain (non-semver) commit message for this tranche.
- [x] Keep roadmap checkbox states aligned with repo reality.

