# 2026-04-14 — Tranche 25: Roadmap clarification (offline load resumability is an implementation detail)

Implemented items (30):

- [x] Clarify that offline-load resumability is driven by the original “resumability” requirement.
- [x] Clarify that staging+swap is an implementation approach, not a new product feature.
- [x] Clarify that truncate+reload is an implementation approach gated by an explicit unsafe flag.
- [x] Clarify that per-table checkpoints is an implementation approach requiring stable keys + line-boundary replay.
- [x] Add “choose exactly one approach” instruction to avoid triple implementation.
- [x] Remove duplicated/ambiguous wording that implied multiple strategies must be built.
- [x] Keep the offline-load resumability checkbox itself pending (not implemented yet).
- [x] Preserve completed states for offline convert resumability (state.json + TSV append).
- [x] Preserve completed states for cache policy items already implemented.
- [x] Preserve completed states for streaming ingestion resumability (bm_meta checkpoints).
- [x] Preserve completed states for indexing-after-load policies.
- [x] Preserve completed states for 1-to-1 MySQL→Postgres mapping.
- [x] Preserve incremental update roadmap states (no changes).
- [x] Keep roadmap organization feature-based (no tranche grouping inside roadmaps).
- [x] Ensure all actionable roadmap lines remain checkboxes.
- [x] Avoid adding any new scope beyond the existing LibGen ingestion requirements.
- [x] Avoid adding manual QA tasks as checkbox items.
- [x] Keep integration-test requirements attached to each approach (still verifiable).
- [x] Keep top-level roadmap aligned with the clarified “choose one approach” rule.
- [x] Avoid modifying config roadmap in this tranche (clarification only).
- [x] Avoid modifying docs/cli or docs/config in this tranche (roadmap-only).
- [x] Avoid modifying code behavior in this tranche.
- [x] Avoid modifying migrations or schema in this tranche.
- [x] Keep tranche log under `docs/tranches/` for traceability.
- [x] Keep tranche checklist count at 30 for process discipline.
- [x] Verify repo remains buildable after roadmap edits.
- [x] Preserve repo auditability: roadmap intent matches original requirements.
- [x] Ensure phrasing explicitly labels A/B/C as “approaches”.
- [x] Ensure phrasing explicitly labels “offline load resumability” as restart-safe without cleanup.
- [x] Provide a plain (non-semver) commit message for this tranche.

