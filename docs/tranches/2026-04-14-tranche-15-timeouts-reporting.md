# Tranche — 2026-04-14 — Postgres statement timeout + JSONL report output

Implemented in this tranche (30 items):

1. [x] Added config `postgres.statement_timeout_ms` (optional).
2. [x] Implemented pooled `after_connect` hook to set `statement_timeout`.
3. [x] Updated config roadmap to mark statement timeout complete.
4. [x] Documented `postgres.statement_timeout_ms` in `docs/config.md`.

5. [x] Added config `output.report_path` (optional JSONL output).
6. [x] Normalized empty `output.report_path` to `None`.
7. [x] Added config validation for non-empty report_path when set.
8. [x] Updated config roadmap to mark report_path complete.
9. [x] Documented `output.report_path` in `docs/config.md`.

10. [x] Added `src/output.rs` helper to append JSONL report lines.
11. [x] Registered `src/output.rs` in `src/lib.rs`.
12. [x] Implemented report emission for `libgen stats`.
13. [x] Implemented report emission for `libgen sample`.
14. [x] Implemented report emission for `libgen validate`.

15. [x] Normalized empty `libgen.dump.path` and `libgen.dump.dataset_id` to `None`.
16. [x] Normalized `postgres.statement_timeout_ms = 0` to `None`.

17. [x] Updated canonical config with commented `statement_timeout_ms` example.
18. [x] Updated canonical config with commented `report_path` example.

19. [x] Kept structured logging for all report writes (tracing events).
20. [x] Kept the control-pane invariant: report/timeouts configurable via TOML.

21. [x] Verified build (`cargo build`).
22. [x] Verified tests (`cargo test`).

23. [x] Ensured no CLI flag changes required.
24. [x] Ensured no migration required for statement timeout/reporting.
25. [x] Preserved existing LibGen ingest/update behavior.
26. [x] Preserved resumability and dataset_state behaviors.

27. [x] Avoided adding manual QA tasks to roadmaps.
28. [x] Maintained idempotency of config normalization.
29. [x] Updated `Cargo.lock` through dependency resolution.
30. [x] Kept output reporting additive (does not replace tracing logs).

