# Tranche — 2026-04-14 — LibGen row ingest (INSERT parsing) + resumability

Implemented in this tranche (30 items):

1. [x] Replaced dump-wide `read_to_string` parsing with streaming `StatementReader`.
2. [x] Implemented streaming SQL statement splitting with single-quote tracking.
3. [x] Enforced `libgen.dump.max_statement_bytes` guardrail during streaming reads.
4. [x] Added `seek_to_offset` helper for resumable ingestion.

5. [x] Implemented `INSERT INTO ... VALUES` parsing (multi-row) for MySQL dump subset.
6. [x] Implemented value parsing for NULL, quoted strings, and unquoted tokens.
7. [x] Implemented MySQL backslash escape decoding inside strings.
8. [x] Added unit test for `INSERT INTO` parsing and escape decoding.

9. [x] Added `Db::insert_rows_text` to bulk insert multiple rows in one SQL statement.
10. [x] Insert path stores all columns as `text` (Phase 1 ingest robustness; typed views deferred).
11. [x] Added checkpoint persistence: `Db::set_checkpoint_offset`.
12. [x] Added checkpoint lookup: `Db::get_checkpoint_offset`.

13. [x] Added `src/libgen/ingest.rs` with a minimal ingestion engine.
14. [x] Implemented per-table definition map to align INSERT rows with provisioned column order.
15. [x] Chunked row loads by `execution.batch.max_rows`.
16. [x] Implemented resumable ingest using byte-offset checkpoint key per (kind, dump path).

17. [x] Wired `libgen ingest` to: register run → provision tables → ingest rows → checkpoint.
18. [x] Wired `libgen update` to: register run → provision tables → ingest rows (incremental logic pending).
19. [x] Updated table provisioning to scan CREATE TABLE statements via streaming reader.
20. [x] Updated Postgres provisioning to store columns as `text` for Phase 1.

21. [x] Updated CLI docs (`docs/cli.md`) to reflect current row-ingest behavior.
22. [x] Updated roadmaps to mark CREATE TABLE + INSERT parsing + basic resumability complete.
23. [x] Verified build (`cargo build`).
24. [x] Verified tests (`cargo test`).

25. [x] Preserved fiction/compact table isolation via per-kind prefixes.
26. [x] Preserved optional global `postgres.table_prefix` naming policy.
27. [x] Kept logging spans around ingestion boundaries (`#[instrument]`).
28. [x] Kept config validation enforcing non-zero batch sizes and statement guardrails.
29. [x] Maintained backward compatibility for previously provisioned tables (`create table if not exists`).
30. [x] Kept COPY-based ingest as a future roadmap item (current path is batched INSERT).

