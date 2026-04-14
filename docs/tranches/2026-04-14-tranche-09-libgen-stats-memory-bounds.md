# Tranche — 2026-04-14 — LibGen stats + bounded-memory ingestion improvements

Implemented in this tranche (30 items):

1. [x] Added config key `execution.memory_hard_limit_bytes`.
2. [x] Added config validation for `execution.memory_hard_limit_bytes > 0`.
3. [x] Updated `config/bulk-merge.toml` with default `execution.memory_hard_limit_bytes`.
4. [x] Updated `docs/roadmaps/CONFIG_ROADMAP.md` to mark memory hard limit complete.
5. [x] Updated `docs/config.md` to document execution limits (memory + batch bounds).

6. [x] Refactored LibGen ingest to avoid buffering an entire INSERT’s rows in memory.
7. [x] Implemented incremental chunk building and flushing for INSERT rows.
8. [x] Enforced `execution.batch.max_rows` during chunk flushing.
9. [x] Enforced `execution.batch.max_bytes` during chunk flushing (approx by string size).
10. [x] Added a safety flush trigger tied to `execution.memory_hard_limit_bytes` (guardrail).

11. [x] Added DB helper `Db::recent_import_runs` to fetch last N libgen runs.
12. [x] Enabled SQLx chrono decoding by adding `sqlx` feature `chrono`.
13. [x] Added `chrono` dependency for typed timestamps in stats output.
14. [x] Added DB helper `Db::raw_statement_count` for raw landing counts.

15. [x] Implemented `bulk-merge libgen stats` command (no longer placeholder).
16. [x] `libgen stats` lists table counts per kind (by prefix).
17. [x] `libgen stats` prints the last 5 `bm_meta.import_run` entries for `source_name='libgen'`.
18. [x] `libgen stats` prints raw statement count per recent import run.
19. [x] Documented `libgen stats` in `docs/cli.md`.

20. [x] Removed unused LibGen command placeholder code after implementing stats.
21. [x] Kept logging spans and structured fields for stats output.
22. [x] Preserved single-schema `src_libgen` strategy and prefix-based isolation.
23. [x] Preserved resumability and checkpoint behavior (unchanged).
24. [x] Preserved raw landing table behavior (unchanged).

25. [x] Updated LibGen ingestion roadmap to mark bounded-memory/backpressure complete.
26. [x] Updated LibGen ingestion roadmap to mark `libgen stats` complete.
27. [x] Ensured compilation after removing placeholder instrumentation.
28. [x] Updated `Cargo.lock` via dependency resolution.
29. [x] Verified tests (`cargo test`).
30. [x] Verified build (`cargo build`).

