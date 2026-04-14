# Tranche — 2026-04-14 — LibGen schema discovery + Postgres table provisioning

Implemented in this tranche (30 items):

1. [x] Added config validation (`AppConfig::validate`) with fail-fast errors.
2. [x] Applied config env overrides: `BULK_MERGE_POSTGRES_URL`, `BULK_MERGE_LOG_LEVEL`, `BULK_MERGE_LOG_FORMAT`.
3. [x] Extended Postgres config surface with `postgres.table_prefix`.
4. [x] Extended LibGen config surface with `libgen.dump.max_statement_bytes` and `libgen.dump.allow_invalid_utf8`.
5. [x] Extended LibGen config surface with `libgen.dump.path` and `libgen.dump.dataset_id`.
6. [x] Added LibGen table naming config: `libgen.tables.fiction_prefix` and `libgen.tables.compact_prefix`.
7. [x] Added LibGen resumability config surface: `libgen.resume.enabled` and `libgen.resume.checkpoint_granularity`.
8. [x] Added LibGen incremental config surface: `libgen.incremental.strategy` and `libgen.incremental.apply_deletes`.
9. [x] Updated `config/bulk-merge.toml` with the new LibGen and Postgres settings.
10. [x] Documented env overrides in `docs/config.md`.

11. [x] Introduced `src/libgen/mysql_dump.rs` for MySQL dump parsing logic.
12. [x] Implemented a minimal SQL statement splitter with single-quote escape handling.
13. [x] Enforced `libgen.dump.max_statement_bytes` guardrail during statement splitting.
14. [x] Implemented `CREATE TABLE` detection (case-insensitive) with comment stripping.
15. [x] Implemented table-name parsing for backtick-quoted and bare identifiers.
16. [x] Implemented top-level column/constraint comma splitting that respects nested parens.
17. [x] Implemented column extraction while skipping key/constraint lines.
18. [x] Added unit tests for `CREATE TABLE` parsing and comma splitting.

19. [x] Introduced `src/libgen/provision.rs` for Postgres provisioning based on schema discovery.
20. [x] Implemented dump scan for `CREATE TABLE` statements and collected `TableDef`s.
21. [x] Selected per-kind table prefixes (`fiction_` vs `compact_`) from config.
22. [x] Applied optional global `postgres.table_prefix` to provisioned tables.
23. [x] Added `Db::create_table_from_def` to create Postgres tables from `TableDef`.
24. [x] Added safe identifier quoting for schema/table/column names.
25. [x] Implemented basic MySQL→Postgres type mapping for common column types.

26. [x] Wired `libgen ingest` to provision tables from dump schema after registering an import run.
27. [x] Wired `libgen update` to provision tables from dump schema after registering an import run.
28. [x] Updated CLI docs (`docs/cli.md`) to reflect current behavior (provisioning only; no row ingest yet).
29. [x] Updated feature roadmaps to reflect completed schema-discovery/provisioning items.
30. [x] Verified build and tests (`cargo build`, `cargo test`).

