# 2026-04-14 — Tranche 19: Roadmap decomposition (cache policy + offline-load resumability)

Implemented items (30):

- [x] Decompose LibGen naming-policy roadmap item into concrete, config-driven sub-items.
- [x] Add `paths.cache_dir` as an explicit dependency for offline artifact placement.
- [x] Add offline artifact layout sub-item (per-kind subdir naming policy).
- [x] Add offline load dataset naming policy sub-item (dataset_id selection when absent).
- [x] Preserve existing checkbox states for completed configuration items.
- [x] Preserve feature-based organization (no tranche-based restructuring of roadmaps).
- [x] Decompose parser guardrails roadmap item into verifiable sub-items (error context, caps, tests).
- [x] Specify bounded statement preview requirement for parse error context.
- [x] Specify typed guardrail error requirement for “statement too large”.
- [x] Specify parser tests that assert guardrails (no panics) requirement.
- [x] Decompose offline load resumability roadmap item into explicit strategy options (A/B/C).
- [x] Add explicit “choose one strategy” framing to avoid building all three.
- [x] Add integration-test requirement for interrupted offline load restart behavior.
- [x] Decompose cache policy roadmap item into concrete CLI defaulting behaviors.
- [x] Keep `--out-dir` override behavior explicitly called out (bypasses cache defaulting).
- [x] Add documentation requirement for cache directory contents and cleanup expectations.
- [x] Add config roadmap `paths.cache_policy` knob as a dependency for cache defaulting behavior.
- [x] Add config roadmap note that `libgen.offline.out_dir_default` should derive from `paths.cache_dir` by default.
- [x] Keep config roadmap aligned with control-pane invariant (tunable policies in TOML).
- [x] Update top-level roadmap with cache policy implementation item (Phase 1 foundation).
- [x] Keep top-level roadmap aligned with actual code state (offline TSV path remains checked).
- [x] Avoid adding unrelated new sources/features while decomposing roadmaps.
- [x] Avoid introducing manual QA tasks as checkbox items (doc tasks remain implementable).
- [x] Ensure all new roadmap items are implementable/verifiable in this repo.
- [x] Ensure all new roadmap items remain scoped to existing approved requirements.
- [x] Keep language concrete and testable (integration test called out where relevant).
- [x] Maintain consistent checkbox formatting (no bullets without checkboxes for actionable work).
- [x] Keep roadmap locations unchanged under `docs/roadmaps/`.
- [x] Add tranche log under `docs/tranches/` for traceability of roadmap changes.
- [x] Verify repo builds after roadmap-only changes.

