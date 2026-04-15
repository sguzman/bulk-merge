# 2026-04-14 — Tranche 31: Offline load safety checks + load-status summary

Roadmap items targeted (verbatim):

- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: Cache policy: all on-disk intermediate artifacts and temp outputs default under `./.cache/bulk-merge/` (configurable root).
- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: `bulk-merge libgen load --import-run-id/--resume-latest` validates the manifest kind/dump matches the import_run config (default strict; configurable override).
- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: `bulk-merge libgen load-status` includes summary counts (staged/swapped/unknown) for a run id in addition to per-table rows.
- `docs/roadmaps/CONFIG_ROADMAP.md`: `libgen.offline.load.resume_strict_manifest_match` (resume safety policy).

Implemented items (roadmap-derived):

- [x] Mark LibGen cache policy top-level checkbox complete (all sub-items were already implemented).
- [x] Add `libgen.offline.load.resume_strict_manifest_match` config knob (default true).
- [x] Add DB helper to fetch `bm_meta.import_run.config_json` for a run id.
- [x] Enforce strict manifest kind/dump match when resuming offline loads via `--import-run-id/--resume-latest` (config-gated).
- [x] Enhance `libgen load-status` to compute stage summary counts and emit them via logs and report JSONL.
- [x] Update `docs/cli.md` to document load-status summary behavior.
- [x] Check off the targeted LibGen roadmap items as completed.
- [x] Check off the targeted config roadmap item as completed.
- [x] Verify `cargo test` passes after changes.

