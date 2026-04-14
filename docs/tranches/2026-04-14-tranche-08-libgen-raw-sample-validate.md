# Tranche — 2026-04-14 — LibGen raw landing + sample/validate operator commands

Implemented in this tranche (30 items):

1. [x] Added migration `migrations/0002_libgen_raw.sql`.
2. [x] Created `src_libgen.raw_statement` raw landing table (statement-level provenance).
3. [x] Added index `idx_raw_statement_mysql_table` for raw statement filtering.

4. [x] Added dependency `sha2` for SHA-256 hashing.
5. [x] Implemented `Db::insert_libgen_raw_statement` (sha256 + upsert-by-offset).
6. [x] Raw statements store: `import_run_id`, `byte_offset_end`, `stmt_kind`, `mysql_table`, `sha256`, `payload`.

7. [x] Added config surface `libgen.raw.enabled`.
8. [x] Added config surface `libgen.raw.store_other_statements`.
9. [x] Updated `config/bulk-merge.toml` with `libgen.raw.*` defaults.
10. [x] Updated `docs/roadmaps/CONFIG_ROADMAP.md` to track LibGen raw landing config.

11. [x] Stored `CREATE TABLE` statements into `src_libgen.raw_statement` during provisioning scan.
12. [x] Stored `INSERT INTO` statements into `src_libgen.raw_statement` during ingest loop.
13. [x] Optionally stored non-INSERT/non-CREATE statements when configured.
14. [x] Ensured all raw statement inserts use the correct `import_run_id` (FK-safe).

15. [x] Added `Db::sample_table` using `row_to_json` for simple sampling.
16. [x] Extended CLI `libgen sample` to require `--kind` and `--mysql-table`.
17. [x] Implemented `libgen sample` to resolve the prefixed Postgres table and fetch sample rows.
18. [x] Added `libgen validate --kind --mysql-table` with a minimal invariant (row count > 0).
19. [x] Updated CLI docs (`docs/cli.md`) to include `sample` and `validate` usage.

20. [x] Added `StatementReader::new_with_offset` for absolute-offset accounting.
21. [x] Wired ingestion to use `StatementReader::new_with_offset` after resume seek.

22. [x] Added `Db::upsert_import_file` for `bm_meta.import_file` path tracking.
23. [x] Added `Db::update_import_file_progress` for `last_offset` + row counters.
24. [x] Kept raw statement offsets consistent with resumability offsets (byte offsets).

25. [x] Updated LibGen ingestion roadmap to mark raw landing/provenance complete.
26. [x] Updated LibGen ingestion roadmap to mark `sample` and `validate` complete.
27. [x] Preserved fiction/compact isolation via table prefixes.
28. [x] Preserved single-schema strategy (`src_libgen`) for all LibGen tables.

29. [x] Verified tests (`cargo test`).
30. [x] Verified build (`cargo build`).

