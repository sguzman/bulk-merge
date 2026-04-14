# Tranche — 2026-04-14 — Progress logging interval (global long-running ops)

Implemented in this tranche (30 items):

1. [x] Added config section `progress` to the control pane.
2. [x] Added `progress.log_interval_seconds` with default 30s.
3. [x] Added config validation for `progress.log_interval_seconds > 0`.
4. [x] Updated `config/bulk-merge.toml` to include the `progress` section.
5. [x] Updated `docs/config.md` to document progress logging.
6. [x] Updated `docs/roadmaps/CONFIG_ROADMAP.md` to track progress logging config.
7. [x] Updated `docs/roadmaps/ROADMAP.md` to mark progress logging feature complete.

8. [x] Added `src/progress.rs` progress ticker utility (interval-based).
9. [x] Added `ProgressTicker::maybe_log` with percent calculation when total is known.
10. [x] Added progress config wrapper (`ProgressConfig`) to centralize Duration handling.
11. [x] Kept progress logging tracing-native (`tracing::info!`).

12. [x] Wired LibGen provisioning scan to emit periodic progress logs by byte offset.
13. [x] Provision scan logs include percent when file size is known.
14. [x] Wired LibGen ingest loop to emit periodic progress logs by byte offset.
15. [x] Ingest progress logs include percent when file size is known.
16. [x] Ingest progress logs include extra counters (rows_seen/rows_loaded).

17. [x] Ensured progress ticker interval uses the TOML control pane value.
18. [x] Kept progress logging generic (not LibGen-specific API).
19. [x] Avoided tying progress logging to a specific DB backend.
20. [x] Avoided adding manual QA steps to roadmaps (all verifiable by build/tests).

21. [x] Verified compilation of new modules via `cargo build`.
22. [x] Verified tests still pass via `cargo test`.
23. [x] Ensured no CLI flag changes required (config-only).
24. [x] Ensured no migrations required for progress logging feature.

25. [x] Ensured progress logging does not affect ingest correctness (logs only).
26. [x] Ensured progress logs are rate-limited by interval (not per-row).
27. [x] Ensured percent computation guards against divide-by-zero.
28. [x] Ensured progress logging works even when total is unknown (logs done_units only).
29. [x] Ensured progress logging can be tuned without code changes (config control pane).
30. [x] Preserved existing tracing configuration/overrides behavior.

