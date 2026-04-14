# Tranche — 2026-04-14 — Incremental update integration test (v1 → v2)

Implemented in this tranche (30 items):

1. [x] Added integration test `tests/libgen_incremental.rs`.
2. [x] Test writes two fixture dumps (v1 and v2) into `tmp/`.
3. [x] v1 fixture ingests IDs 1 and 2.
4. [x] v2 fixture updates ID 1, deletes ID 2, inserts ID 3.

5. [x] Test uses a unique `postgres.table_prefix` per run to avoid clobbering real data.
6. [x] Test runs migrations before exercising ingestion/update.
7. [x] Test provisions tables from dump schema for both v1 and v2.

8. [x] Test executes v1 ingest using `IngestMode::Ingest`.
9. [x] Test executes v2 update using `IngestMode::Update`.
10. [x] Test ensures unique index exists for the PK before update upserts.
11. [x] Test applies delete handling via `delete_rows_not_seen`.

12. [x] Test asserts ID=1 title is updated.
13. [x] Test asserts ID=2 is deleted.
14. [x] Test asserts ID=3 exists.

15. [x] Test asserts dataset_state points to the v2 run/version.
16. [x] Test exercises `Db::get_text_by_pk`.
17. [x] Test exercises `Db::get_dataset_state`.

18. [x] Added `Db::get_text_by_pk` helper for targeted assertions.
19. [x] Added `Db::get_dataset_state` helper for dataset_state assertions.

20. [x] Added `uuid` as a dev-dependency for unique test prefixes.
21. [x] Updated LibGen ingestion roadmap to check off incremental tests.

22. [x] Verified tests (`cargo test`).
23. [x] Verified build (`cargo build`).

24. [x] Preserved streaming parsing (fixtures still go through statement reader).
25. [x] Preserved Phase 1 text storage semantics.
26. [x] Preserved apply_deletes single-column PK limitation.
27. [x] Avoided adding manual QA steps.
28. [x] Kept config control pane as the source of DB connection settings.
29. [x] Updated `Cargo.lock` through dependency resolution.
30. [x] Kept roadmap feature-based (tranche recorded separately here).

