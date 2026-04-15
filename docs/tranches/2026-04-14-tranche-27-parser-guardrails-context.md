# 2026-04-14 — Tranche 27: Parser guardrails + contextual errors

Implemented items (30):

- [x] Extend `MySqlDumpError::StatementTooLarge` to include current bytes, offset_end, and a bounded preview.
- [x] Ensure statement-size guardrail triggers before unbounded buffering occurs.
- [x] Add `libgen.dump.error_preview_bytes` config knob for parse error previews.
- [x] Validate `libgen.dump.error_preview_bytes` is > 0.
- [x] Add canonical TOML setting `libgen.dump.error_preview_bytes = 256`.
- [x] Document parsing guardrails in `docs/config.md`.
- [x] Add `statement_preview(...)` helper for consistent bounded previews.
- [x] Add a unit test asserting `StatementTooLarge` includes context fields (no panics).
- [x] Add contextual parse errors for `INSERT INTO` in streaming ingest (offset_end + preview).
- [x] Add contextual parse errors for `CREATE TABLE` in provisioning scan (offset_end + preview).
- [x] Add contextual parse errors for `CREATE TABLE` in offline conversion (offset_end + preview).
- [x] Add contextual parse errors for `INSERT INTO` in offline conversion (offset_end + preview).
- [x] Keep memory hard limit behavior unchanged for streaming ingestion batches.
- [x] Keep max statement size enforcement as an explicit guardrail.
- [x] Keep statement splitting semantics unchanged (quotes + backslash escapes).
- [x] Avoid any whole-dump buffering (remain streaming).
- [x] Keep resumability behavior unchanged (byte offsets in bm_meta / state.json).
- [x] Keep post-load indexing policy unchanged.
- [x] Keep MySQL→Postgres 1-to-1 mapping unchanged.
- [x] Keep libgen update incremental behavior unchanged.
- [x] Update LibGen roadmap guardrails checkbox and sub-items as completed.
- [x] Update config roadmap to include `libgen.dump.error_preview_bytes`.
- [x] Preserve other roadmap checkbox states.
- [x] Avoid adding new crates for guardrails (use existing `thiserror`).
- [x] Ensure error previews are bounded and do not log entire giant statements.
- [x] Ensure guardrail errors remain typed (matchable in tests).
- [x] Ensure new config knobs honor the control-pane invariant.
- [x] Verify build and tests pass after changes.
- [x] Record tranche under `docs/tranches/` for auditability.
- [x] Provide a plain (non-semver) commit message for this tranche.

