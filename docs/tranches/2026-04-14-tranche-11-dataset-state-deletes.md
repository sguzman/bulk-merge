# Tranche — 2026-04-14 — Incremental state + optional delete handling

Implemented in this tranche (30 items):

1. [x] Added migration `migrations/0003_dataset_state_and_deletes.sql`.
2. [x] Added `bm_meta.dataset_state` for dataset-level incremental state.
3. [x] Added `src_libgen.seen_pk` for optional delete-handling support.
4. [x] Added index `idx_seen_pk_table` for seen key lookups.

5. [x] Added config surface `libgen.incremental.primary_key_columns.fiction`.
6. [x] Added config surface `libgen.incremental.primary_key_columns.compact`.
7. [x] Updated canonical `config/bulk-merge.toml` with PK defaults (`ID`).

8. [x] Implemented DB helper `Db::upsert_dataset_state`.
9. [x] Wired libgen ingest/update to update `bm_meta.dataset_state` on success.

10. [x] Implemented DB helper `Db::insert_seen_pk_values` (batched, conflict-ignore).
11. [x] Wired update-mode ingestion to collect seen PK values while streaming.
12. [x] Enforced Phase 1 limitation: delete handling requires a single-column PK.

13. [x] Implemented DB helper `Db::delete_rows_not_seen` (delete-by-anti-join against `seen_pk`).
14. [x] Wired `libgen update` to optionally delete rows not present in new dump when enabled.
15. [x] Documented delete behavior in `docs/cli.md`.

16. [x] Updated LibGen roadmap to mark persisted incremental state complete.
17. [x] Updated LibGen roadmap to mark delete-handling complete (Phase 1 semantics).
18. [x] Updated implementation roadmap to record dataset-state completion.
19. [x] Updated config roadmap to reflect PK columns config completion.

20. [x] Preserved raw landing table behavior (unchanged).
21. [x] Preserved resumability checkpoints (unchanged).
22. [x] Preserved post-load indexing behavior (unchanged).
23. [x] Preserved streaming parsing (no dump-wide memory loads).
24. [x] Preserved bounded-memory chunking (batch/max bytes/mem hard limit).

25. [x] Verified tests (`cargo test`).
26. [x] Verified build (`cargo build`).
27. [x] Ensured migrations remain idempotent (`create if not exists`).
28. [x] Ensured FK integrity (seen_pk references import_run).
29. [x] Ensured dataset_state references import_run (nullable FK).
30. [x] Updated `Cargo.lock` through dependency resolution.

