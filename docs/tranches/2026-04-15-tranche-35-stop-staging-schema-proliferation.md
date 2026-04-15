# 2026-04-15 — Tranche 35: Stop staging schema proliferation (single schema policy)

Roadmap items targeted (verbatim):

- `docs/roadmaps/CONFIG_ROADMAP.md`: `libgen.offline.load.*` (offline load strategy + staging/swap policy knobs).

Implemented items (roadmap-derived):

- [x] Replace per-run staging schemas with per-run staging *tables* inside the live LibGen schema to avoid proliferating schemas.
- [x] Add `libgen.offline.load.staging_table_suffix_template` to control staging table naming.
- [x] Replace `drop_staging_schema_on_success` with `drop_staging_tables_on_success` for cleanup within the single-schema strategy.

Build verification:

- `cargo test`

