# 2026-04-14 — Tranche 17: Cache dir defaults

Implemented items (30):

- [x] Change LibGen offline default output directory to `.cache/bulk-merge/libgen-offline`.
- [x] Preserve CLI override (`--out-dir`) as highest-precedence output location.
- [x] Preserve resumability behavior (append TSV + `state.json` checkpoint) under the new default cache path.
- [x] Keep the offline artifact layout stable (`manifest.json`, `state.json`, per-table `*.tsv`).
- [x] Keep `.cache` ignored via existing `.gitignore` entry.
- [x] Update `config/bulk-merge.toml` to reflect the new default offline output directory.
- [x] Update `docs/config.md` to document the new default cache path.
- [x] Update `docs/cli.md` to document the default `--out-dir` behavior.
- [x] Ensure no new “hidden” temp files are introduced outside configured offline output directories.
- [x] Confirm streaming ingest resumability remains DB-backed (`bm_meta.import_checkpoint`) with no filesystem temp files.
- [x] Confirm streaming ingest continues to use bounded batching (rows/bytes) with `execution.memory_hard_limit_bytes`.
- [x] Confirm offline load continues to stream TSV bytes in chunks into Postgres COPY (no whole-file buffering).
- [x] Keep COPY settings consistent between online and offline paths (CSV format, tab delimiter, `\\N` NULL, quote/escape `"`).
- [x] Preserve post-load indexing policy (`postgres.indexing.create_after_load`) for offline load.
- [x] Preserve 1-to-1 column mapping policy (MySQL columns → Postgres `text`) for offline load.
- [x] Avoid changing schema/table naming policies while adjusting cache directory defaults.
- [x] Avoid adding new dependencies while adjusting cache directory defaults.
- [x] Avoid changing libgen incremental update strategy while adjusting cache directory defaults.
- [x] Avoid changing bm_meta bookkeeping while adjusting cache directory defaults.
- [x] Avoid changing raw landing behavior while adjusting cache directory defaults.
- [x] Keep config validation behavior unchanged (only default values changed).
- [x] Keep observability behavior unchanged (only docs/defaults changed).
- [x] Keep any existing offline outputs working if `libgen.offline.out_dir_default` is explicitly set by operator.
- [x] Keep compatibility with non-default working directories (relative `.cache/...` resolves under CWD).
- [x] Ensure convert still `create_dir_all` for the output directory.
- [x] Ensure manifest/state writes remain atomic-per-write (overwrite whole JSON file).
- [x] Ensure no changes are required to existing tests for this default change.
- [x] Ensure `cargo test` remains clean after default change.
- [x] Ensure project remains buildable after default change.
- [x] Keep the config control-pane invariant: default can be overridden in TOML.

