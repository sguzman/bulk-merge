# Tranche — 2026-04-13 — Foundations (CLI + config + logging + bm_meta)

Implemented in this tranche (30 items):

1. [x] Added `clap`-based CLI entrypoint with `bulk-merge` name/version/help.
2. [x] Added global CLI flags: `--config`, `--dry-run`, `--log-level`, `--log-format`.
3. [x] Added `init-db` command that connects to Postgres and runs migrations.
4. [x] Added `libgen ingest` command stub that registers an `import_run` (no data load yet).
5. [x] Added `libgen update` command stub that registers an `import_run` (no data load yet).
6. [x] Added `libgen stats` command placeholder (connects + migrates; no stats yet).
7. [x] Added `libgen sample` command placeholder (connects + migrates; no sampling yet).
8. [x] Added `libgen validate` command placeholder (connects + migrates; no validation yet).
9. [x] Added structured `tracing` spans around command boundaries (`#[instrument]`).
10. [x] Added `tracing-subscriber` initialization with env-filter driven log levels.
11. [x] Added log format selection (`human` vs `json`) via config/CLI override.
12. [x] Added TOML configuration model (`AppConfig`) with Postgres/logging/execution/libgen sections.
13. [x] Implemented config loader from disk (`toml` + `serde`).
14. [x] Added canonical control-pane file at `config/bulk-merge.toml`.
15. [x] Set default Postgres URL to match `tmp/docker-compose.yaml` (admin/admin, db `data`, host `127.0.0.1:5432`).
16. [x] Added Postgres pool knobs to config (`max_connections`, `min_connections`, `acquire_timeout_ms`).
17. [x] Added execution policy knobs to config (`dry_run_default`, `concurrency`, batching, retries).
18. [x] Added LibGen policy knobs to config (dump kind, resume/incremental placeholders, statement-size guardrail placeholder).
19. [x] Added Postgres indexing policy knobs to config (`create_after_load`, `concurrent`).
20. [x] Added `sqlx` Postgres pool connector (`Db::connect`).
21. [x] Added migration runner (`Db::migrate`) wired to `./migrations`.
22. [x] Added initial migration `migrations/0001_init.sql`.
23. [x] Migration creates `bm_meta` schema.
24. [x] Migration creates `bm_meta.import_run`.
25. [x] Migration creates `bm_meta.import_file`.
26. [x] Migration creates `bm_meta.import_checkpoint`.
27. [x] Migration creates `src_libgen` schema (Phase 1 namespace).
28. [x] Added `Db::create_import_run` for `bm_meta.import_run`.
29. [x] Added `Db::finish_import_run` to close out an import run.
30. [x] Added baseline docs: `README.md`, `docs/cli.md`, `docs/config.md`.

Build verification:

- [x] `cargo build`

