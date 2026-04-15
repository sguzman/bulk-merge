# 2026-04-15 — Tranche 34: Best-effort typed LibGen columns + reset

Roadmap items targeted (verbatim):

- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: Map LibGen table columns 1-to-1 from the MySQL dump into PostgreSQL columns (Phase 1: best-effort typed mapping; invalid values become NULL).
- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: `bulk-merge libgen reset --kind {fiction|compact|all}` drops derived LibGen tables so they can be recreated with new policies (e.g. typing changes).
- `docs/roadmaps/CONFIG_ROADMAP.md`: `libgen.typing.mode` (best-effort typed columns vs all-text).
- `docs/roadmaps/CONFIG_ROADMAP.md`: `libgen.typing.unrepresentable_policy` (what to do when a value can't be coerced to the target type).

Implemented items (roadmap-derived):

- [x] Provision typed Postgres columns from MySQL `CREATE TABLE` types when `libgen.typing.mode=best_effort`.
- [x] Best-effort value coercion during ingest/update; unrepresentable values follow `libgen.typing.unrepresentable_policy` (default: NULL).
- [x] Best-effort coercion during offline TSV conversion so `COPY` into typed columns doesn't fail; invalid values become `\\N` (NULL) by default.
- [x] Typed-safe `INSERT`/`UPSERT` paths for the `insert` loader by inserting from `(VALUES ...) AS v(...)` with explicit casts per column.
- [x] Fix delete/apply-delete bookkeeping queries to compare primary keys via `::text` so typed PK columns work with `seen_pk.pk_value` storage.
- [x] Add `bulk-merge libgen reset --kind {fiction|compact|all}` for dropping derived tables to enable clean re-ingest with new typing policy.
- [x] Update roadmaps to reflect completed typing/reset/config items.

Build verification:

- `cargo test`

