# Tranche — 2026-04-14 — Import file accounting + post-load indexes

Implemented in this tranche (30 items):

1. [x] Added `postgres.indexing.main_fields.fiction` config list.
2. [x] Added `postgres.indexing.main_fields.compact` config list.
3. [x] Updated canonical `config/bulk-merge.toml` with example main fields lists.
4. [x] Updated config roadmap to mark `postgres.indexing.main_fields` complete.

5. [x] Added `Db::upsert_import_file` to create/update `bm_meta.import_file` for a dump path.
6. [x] Added `Db::update_import_file_progress` to track offset/records/status during ingest.
7. [x] Wired LibGen ingestion to create an `import_file` row (with file size when available).
8. [x] Wired LibGen ingestion to update `import_file.last_offset` on checkpoint updates.
9. [x] Wired LibGen ingestion to update `import_file.records_seen` and `records_loaded`.
10. [x] Wired LibGen ingestion to mark `import_file` status `succeeded` on completion.

11. [x] Implemented `Db::index_exists` using `pg_class`/`pg_namespace`.
12. [x] Implemented `Db::ensure_btree_index` for post-load index creation (btree).
13. [x] Implemented concurrent index creation path when configured.
14. [x] Wired `libgen ingest/update` to create indexes after row ingest when enabled.
15. [x] Indexes are created only for configured fields that exist in the discovered table schema.

16. [x] Fixed statement offsets to be absolute by adding `StatementReader::new_with_offset`.
17. [x] Ensured resume seeks to a byte offset and continues with absolute offsets.
18. [x] Added unit test for `StatementReader` basic behavior.

19. [x] Updated LibGen ingestion roadmap to mark per-file accounting complete.
20. [x] Updated LibGen ingestion roadmap to mark post-load indexing (streaming path) complete.
21. [x] Updated implementation roadmap to mark post-load indexing complete.
22. [x] Updated CLI docs to reflect post-load indexing behavior.

23. [x] Verified unit tests (`cargo test`).
24. [x] Verified build (`cargo build`).

25. [x] Preserved single-schema `src_libgen` strategy (isolation via prefixes).
26. [x] Preserved resumability checkpoints in `bm_meta.import_checkpoint`.
27. [x] Preserved config validation requirements (non-zero limits).
28. [x] Kept post-load indexing optional via config.
29. [x] Kept COPY-based ingestion as a future roadmap item.
30. [x] Kept index naming deterministic (`idx_<table>_<column>`).

