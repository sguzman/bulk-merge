# 2026-04-14 — Tranche 30: init-db schema provisioning + offline dataset naming policy

Roadmap items targeted (verbatim):

- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: `bulk-merge init-db` provisions LibGen kind-specific tables (once schema discovery exists).
- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: TOML config includes full schema/table naming policy for LibGen provisioned tables.
  - Configurable offline artifact layout under cache dir (per-kind subdir naming policy).
  - Configurable dataset naming policy for offline load (how `dataset_id` is chosen when absent).

Implemented items (roadmap-derived):

- [x] Add `libgen.init.*` config to drive optional `init-db` table provisioning from dump schema discovery.
- [x] Implement schema discovery reuse as `discover_table_defs_from_dump(...)` (no import_run required).
- [x] Wire `init-db` to optionally provision fiction/compact tables from configured dump paths (no row ingest).
- [x] Add `libgen.offline.load.dataset_id_template` config to control dataset_id defaulting (supports `{kind}`).
- [x] Apply dataset_id template in `bulk-merge libgen load` when dataset_id isn’t provided.
- [x] Update `config/bulk-merge.toml` and `docs/config.md` for new config knobs.
- [x] Update `docs/cli.md` to document optional `init-db` provisioning.
- [x] Check off the targeted LibGen roadmap items as completed.
- [x] Check off corresponding config roadmap items as completed.
- [x] Verify `cargo test` passes after changes.

