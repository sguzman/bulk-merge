# 2026-04-14 — Tranche 24: Roadmap design for offline-load resumability

Implemented items (30):

- [x] Decompose offline-load resumability into concrete sub-items for Strategy A (staging + swap).
- [x] Specify a staging schema naming policy as an explicit checkbox item.
- [x] Specify staging table creation requirements (column parity, `_bm_row_hash` parity).
- [x] Specify staging load requirements (COPY + post-load indexing).
- [x] Specify swap requirements (transactional renames + optional keep-old for rollback).
- [x] Specify swap progress persistence requirement in `bm_meta`.
- [x] Specify strategy-specific integration test for “restart after staging load, finish swap”.
- [x] Decompose Strategy B (truncate+reload) into explicit “unsafe” flag + transactional truncate + deterministic reload.
- [x] Specify strategy-specific integration test for truncate+reload interruption/restart.
- [x] Decompose Strategy C (per-table checkpoints) into explicit checkpoint file + line-boundary replay requirement.
- [x] Call out need for resumable COPY mechanics (line-boundary seek) for Strategy C.
- [x] Specify strategy-specific integration test for per-table checkpoint resume without duplication.
- [x] Keep the “choose one strategy” framing to avoid triple implementation.
- [x] Keep the general “interrupted offline load restart” test requirement in place.
- [x] Update top-level roadmap with concrete offline-load resumability sub-items.
- [x] Preserve existing checkbox states across other roadmap sections.
- [x] Avoid adding unrelated new roadmap items outside existing requirements.
- [x] Avoid adding manual QA as checkbox items (all are implementable/verifiable).
- [x] Keep feature-based organization intact (no tranche grouping inside roadmaps).
- [x] Keep roadmap language concrete and testable.
- [x] Maintain strict checkbox formatting discipline for actionable items.
- [x] Keep LibGen roadmap scope limited to LibGen ingestion concerns.
- [x] Keep config roadmap untouched in this tranche (design-only for offline-load resumability).
- [x] Avoid changing code behavior in this roadmap-only tranche.
- [x] Avoid changing config defaults in this roadmap-only tranche.
- [x] Avoid changing docs/cli or docs/config in this roadmap-only tranche.
- [x] Record tranche changes under `docs/tranches/` for auditability.
- [x] Verify repo remains buildable after roadmap edits.
- [x] Keep tranche checklist count at 30.
- [x] Provide a plain (non-semver) commit message for this tranche.

