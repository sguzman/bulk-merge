# Tranche — 2026-04-14 — COPY-based ingest loader (streaming)

Implemented in this tranche (30 items):

1. [x] Added config `execution.loader.kind` (`copy|insert`) to the control pane.
2. [x] Defaulted loader kind to `copy`.
3. [x] Updated `config/bulk-merge.toml` with `execution.loader.kind = "copy"`.
4. [x] Updated config roadmap to mark loader kind complete.
5. [x] Updated `docs/config.md` to document `execution.loader.kind`.

6. [x] Implemented `Db::copy_rows_text_tsv` using `COPY FROM STDIN`.
7. [x] COPY uses CSV format with tab delimiter and NULL `\\N`.
8. [x] COPY implementation quotes non-NULL fields and doubles quotes for escaping.
9. [x] COPY implementation streams bytes (not strings) to PgCopyIn.
10. [x] COPY path acquires a connection from the pool and uses `copy_in_raw`.

11. [x] Wired LibGen ingest to use COPY when `execution.loader.kind = "copy"`.
12. [x] Preserved fallback loader `insert` for environments where COPY is undesirable.
13. [x] Ensured LibGen update continues using upsert (`ON CONFLICT`) and does not attempt COPY.

14. [x] Kept bounded-memory chunking controls (`batch.max_rows`, `batch.max_bytes`, `memory_hard_limit_bytes`) around COPY flushes.
15. [x] Kept resumability checkpoints and import_file progress updates intact.

16. [x] Updated CLI docs to reflect COPY behavior and update-mode distinction.
17. [x] Updated implementation roadmap wording to remove “COPY pending” from streaming ingest.
18. [x] Updated LibGen roadmap wording to reflect COPY-enabled ingest.

19. [x] Fixed compilation errors discovered by `cargo test` during COPY integration.
20. [x] Ensured no dump-wide memory loads were introduced.

21. [x] Verified unit tests (`cargo test`).
22. [x] Verified build (`cargo build`).

23. [x] Preserved raw statement landing behavior.
24. [x] Preserved post-load index creation behavior.
25. [x] Preserved table provisioning semantics.

26. [x] Kept COPY statement deterministic and schema-qualified.
27. [x] Kept identifier quoting consistent with existing DB helpers.
28. [x] Maintained type policy: all ingested columns remain `text` in Phase 1.
29. [x] Avoided adding non-configurable magic numbers; loader choice is in TOML.
30. [x] Updated `Cargo.lock` through dependency resolution.

