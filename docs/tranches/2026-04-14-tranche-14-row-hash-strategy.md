# Tranche — 2026-04-14 — Row-hash de-dupe strategy (optional)

Implemented in this tranche (30 items):

1. [x] Added config `libgen.incremental.row_hash.enabled`.
2. [x] Added config `libgen.incremental.row_hash.algorithm` (sha256).
3. [x] Wired config roadmap to mark row-hash config complete.
4. [x] Updated canonical `config/bulk-merge.toml` with row-hash config defaults.
5. [x] Updated `docs/config.md` to document row-hash strategy semantics.
6. [x] Updated `docs/cli.md` to document row-hash update behavior caveat.

7. [x] Extended Postgres table provisioning to optionally add `_bm_row_hash text not null`.
8. [x] Row-hash column inclusion is enabled only when `strategy="row_hash"` and `row_hash.enabled=true`.
9. [x] Preserved 1-to-1 mapping by keeping original columns unchanged (extra meta column only).

10. [x] Added DB helper `insert_rows_text_on_conflict_do_nothing`.
11. [x] Insert+dedupe validates conflict columns exist in the column list.
12. [x] Insert+dedupe uses deterministic SQL (`ON CONFLICT (...) DO NOTHING`).

13. [x] Added `row_hash_enabled` to LibGen `IngestPlan`.
14. [x] Implemented SHA-256 row-hash computation over row values (NULL as `\\N`, tab delimiter).
15. [x] Appended `_bm_row_hash` value to the row in row-hash mode.
16. [x] Wired ingest to de-dupe by `_bm_row_hash` when row-hash mode is enabled.
17. [x] Wired update to de-dupe by `_bm_row_hash` when row-hash mode is enabled.

18. [x] Ensured unique index exists for the row-hash conflict column.
19. [x] Reused existing unique-index helper for row-hash uniqueness.

20. [x] Added `hex` dependency for row-hash hex encoding.
21. [x] Fixed integration test structs to include new `IngestPlan` field.

22. [x] Cleaned up a duplicated unchecked incremental strategy line in the LibGen roadmap.

23. [x] Verified build (`cargo build`).
24. [x] Verified tests (`cargo test`).

25. [x] Preserved COPY-based ingest for non-row-hash mode.
26. [x] Preserved upsert-by-PK behavior for primary-key strategy.
27. [x] Ensured apply_deletes is disabled when row-hash mode is enabled.
28. [x] Ensured resumability and import_file progress tracking remain intact.
29. [x] Updated `Cargo.lock` through dependency resolution.
30. [x] Maintained feature-based roadmaps with tranche record kept separate.

