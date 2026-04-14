# Tranche — 2026-04-14 — LibGen incremental update (upsert) + run status

Implemented in this tranche (30 items):

1. [x] Added config `libgen.incremental.primary_key_columns.fiction`.
2. [x] Added config `libgen.incremental.primary_key_columns.compact`.
3. [x] Updated canonical `config/bulk-merge.toml` with default PK columns (`ID`).
4. [x] Updated config roadmap to mark `libgen.incremental.primary_key_columns` complete.

5. [x] Added DB helper `Db::upsert_rows_text` using `ON CONFLICT` upsert.
6. [x] Validated conflict columns are non-empty and present in the column list.
7. [x] Added DB helper `Db::ensure_unique_index` for conflict columns.
8. [x] Ensured deterministic unique index naming (`uidx_<table>_<col1>_<col2>...`).

9. [x] Added `IngestMode` to LibGen ingestion engine (ingest vs update).
10. [x] Added `conflict_columns` to `IngestPlan`.
11. [x] Wired `libgen update` to require configured PK columns for the selected kind.
12. [x] Wired `libgen update` to ensure unique indexes exist before upserts.
13. [x] Wired `libgen update` to upsert rows instead of inserting rows.
14. [x] Kept `libgen ingest` behavior as insert-only.

15. [x] Marked import runs as `succeeded` on successful completion (instead of `pending`).
16. [x] Preserved existing tracing spans and structured logs.
17. [x] Preserved fiction/compact isolation via prefixes.

18. [x] Updated LibGen ingestion roadmap to mark ingest and update complete (Phase 1 semantics).
19. [x] Updated implementation roadmap to mark `update libgen` complete (Phase 1 upsert).
20. [x] Updated CLI docs to describe update semantics (full scan + upsert).

21. [x] Verified tests (`cargo test`).
22. [x] Verified build (`cargo build`).

23. [x] Kept delete-handling out of scope (apply_deletes remains unimplemented).
24. [x] Kept persisted incremental state out of scope (dataset-level state pending).
25. [x] Avoided reading dump into memory (streaming statement parsing preserved).
26. [x] Preserved bounded-memory chunking behavior.
27. [x] Preserved post-load indexing behavior for main fields.
28. [x] Preserved raw statement landing behavior.
29. [x] Updated `Cargo.lock` via dependency resolution.
30. [x] Maintained backwards compatibility for `postgres.url` override and prior configs.

