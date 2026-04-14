# 2026-04-14 — Tranche 20: Config-driven cache directory policy

Implemented items (30):

- [x] Add `[paths]` section to the TOML control pane.
- [x] Add `paths.cache_dir` config with default `./.cache/bulk-merge`.
- [x] Add `paths.cache_policy` config (`always|prefer|never`).
- [x] Normalize `paths.cache_dir` to remove trailing slashes.
- [x] Validate `paths.cache_dir` is non-empty.
- [x] Make `libgen.offline.out_dir_default` optional (`Option<String>`).
- [x] Derive `libgen.offline.out_dir_default` from `paths.cache_dir` when omitted (unless `paths.cache_policy="never"`).
- [x] Treat empty `libgen.offline.out_dir_default` as unset during normalization.
- [x] Preserve `--out-dir` as the highest-precedence override for offline conversion.
- [x] Make `bulk-merge libgen convert` error clearly when `paths.cache_policy="never"` and no output dir is provided.
- [x] Keep offline conversion resumability semantics unchanged (`state.json` offset + TSV append).
- [x] Keep offline load streaming semantics unchanged (chunked `COPY FROM STDIN`).
- [x] Keep streaming ingest/update resumability DB-backed (`bm_meta.import_checkpoint`) with no filesystem temp files.
- [x] Update canonical config `config/bulk-merge.toml` with `[paths]` defaults.
- [x] Update canonical config comments to explain derived offline output dir behavior.
- [x] Update `docs/config.md` to document `paths.cache_dir` and `paths.cache_policy`.
- [x] Update `docs/config.md` to document derived `libgen.offline.out_dir_default` behavior.
- [x] Update `docs/cli.md` to document `paths.*` influence on default offline output directory.
- [x] Check off `paths.cache_dir` and `paths.cache_policy` items in `docs/roadmaps/CONFIG_ROADMAP.md`.
- [x] Check off `paths.cache_dir` sub-item in `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`.
- [x] Preserve the config-control-pane invariant (paths/policies are centralized in TOML).
- [x] Avoid introducing new magic paths elsewhere in the codebase.
- [x] Avoid changing database schemas or migrations in this tranche.
- [x] Avoid changing LibGen incremental update behavior in this tranche.
- [x] Avoid changing indexing-after-load behavior in this tranche.
- [x] Avoid changing MySQL→Postgres field mapping behavior in this tranche.
- [x] Keep tracing instrumentation intact for cache-related behavior (errors include context).
- [x] Keep backwards compatibility: explicit `libgen.offline.out_dir_default` in TOML still works.
- [x] Ensure project builds and tests pass after config model changes.
- [x] Record tranche work for auditability under `docs/tranches/`.

