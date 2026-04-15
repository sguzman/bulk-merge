# 2026-04-14 — Tranche 26: Offline load resumability (staging + swap)

Implemented items (30):

- [x] Add TOML config for offline load strategy (`libgen.offline.load.strategy`).
- [x] Add TOML config for staging schema prefix (`libgen.offline.load.staging_schema_prefix`).
- [x] Add TOML config for keeping old tables on swap (`libgen.offline.load.keep_old_tables`).
- [x] Add TOML config for dropping old tables on success (`libgen.offline.load.drop_old_tables_on_success`).
- [x] Add migration `bm_meta.offline_swap_progress` to persist per-table staging/swap progress.
- [x] Add `Db::ensure_schema` helper for creating staging schemas.
- [x] Add `Db::table_exists` helper for guarded DDL decisions.
- [x] Add `Db::drop_table_if_exists` helper for optional cleanup of old tables.
- [x] Add `Db::upsert_offline_swap_progress` to persist staged/swapped state.
- [x] Add `Db::get_offline_swap_stage` to resume staging/swap on restart.
- [x] Add `Db::swap_table_from_staging` to atomically move staging tables into the live schema.
- [x] Implement offline load strategy `staging_swap`: stage via `COPY`, index post-load, then rename/swap into live schema.
- [x] Ensure staging uses the same table names as live tables (only schema differs).
- [x] Ensure indexes are created after staging COPY to preserve “insert fast first” policy.
- [x] Preserve 1-to-1 MySQL→Postgres column mapping (still `text` columns) during staging.
- [x] Ensure staging schema is per-import-run (`<prefix>_<run_id>`) to avoid collisions.
- [x] Ensure swap step persists progress so restart can resume without re-copying completed tables.
- [x] Keep optional rollback trail by renaming old live tables to `${table}__old_<run_id>` when enabled.
- [x] Add optional cleanup path to drop kept old tables after success (config-gated).
- [x] Update offline load entrypoint signature to accept `import_run_id`.
- [x] Wire CLI `bulk-merge libgen load` to pass `import_run_id` into offline load.
- [x] Add a test-only interrupt hook to simulate staging interruption and verify restart behavior.
- [x] Add integration test `tests/libgen_offline_load_resumable.rs` covering interrupt → restart → success.
- [x] Update `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md` to mark offline load resumability completed (Approach A).
- [x] Update `docs/roadmaps/CONFIG_ROADMAP.md` to mark `libgen.offline.load.*` surface completed.
- [x] Update `docs/roadmaps/ROADMAP.md` to mark offline load resumability completed (including integration test).
- [x] Keep streaming ingest/update behavior unchanged.
- [x] Keep cache policy behavior unchanged.
- [x] Verify build and tests pass after changes.
- [x] Record tranche under `docs/tranches/` for auditability.

