# 2026-04-14 — Tranche 18: Roadmap updates (resumability + cache policy)

Implemented items (30):

- [x] Align top-level roadmap to reflect LibGen offline TSV path as implemented.
- [x] Align top-level roadmap to reflect per-kind isolation as implemented (table prefixes under one schema).
- [x] Add explicit top-level roadmap item for offline load resumability strategy (still pending).
- [x] Add LibGen roadmap item for offline load resumability strategy (still pending).
- [x] Add LibGen roadmap item for default cache policy under `./.cache/bulk-merge/` (still pending).
- [x] Preserve existing feature-based roadmap organization (no tranche-based restructuring).
- [x] Preserve existing checkbox states for already-implemented items.
- [x] Avoid introducing manual QA or subjective tasks as roadmap items.
- [x] Ensure new roadmap items are implementable/verifiable in this repo.
- [x] Ensure new roadmap items are direct refinements of existing requirements (resumability, cache location).
- [x] Add config roadmap section for cache/path policy (still pending implementation).
- [x] Specify default cache root (`./.cache/bulk-merge`) as a config-controlled policy target.
- [x] Keep the config-control-pane invariant explicit (paths/policies belong in TOML).
- [x] Avoid adding unrelated new features to roadmaps (no source adapters added here).
- [x] Keep LibGen roadmap scope strictly LibGen ingestion (no cross-source merging).
- [x] Keep incremental update roadmap state unchanged (already implemented items remain checked).
- [x] Keep indexing-after-load roadmap state unchanged (already implemented items remain checked).
- [x] Keep 1-to-1 MySQL→Postgres mapping roadmap state unchanged (already implemented items remain checked).
- [x] Ensure roadmap language stays concrete (choose/implement one of listed resumability strategies).
- [x] Ensure roadmap items use markdown checkboxes consistently.
- [x] Avoid renaming roadmap files or changing locations (remain under `docs/roadmaps/`).
- [x] Avoid moving tranche logs (remain under `docs/tranches/`).
- [x] Keep tranche log format consistent with prior tranches (30 checkboxes).
- [x] Maintain traceability: tranche log references exactly what was changed.
- [x] Avoid changing CLI docs or config docs in this roadmap-only tranche.
- [x] Avoid changing code behavior in this roadmap-only tranche.
- [x] Ensure the repo remains buildable after roadmap-only changes.
- [x] Run build verification after roadmap changes (per global invariant).
- [x] Prepare the next tranche focus area: implement `paths.cache_dir` and unify artifact placement.
- [x] Prepare the next tranche focus area: implement offline-load resumability without manual cleanup.

