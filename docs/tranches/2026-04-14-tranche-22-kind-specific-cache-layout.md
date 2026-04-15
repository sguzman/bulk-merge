# 2026-04-14 — Tranche 22: Kind-specific cache layout for offline artifacts

Implemented items (30):

- [x] Add `libgen.offline.layout` config knob (`kind_subdir|flat`).
- [x] Default `libgen.offline.layout` to `kind_subdir`.
- [x] Keep `libgen.offline.out_dir_default` optional and cache-derived when omitted.
- [x] Make `bulk-merge libgen convert` default to `${out_dir_default}/{fiction|compact}` when no `--out-dir` is provided and layout is `kind_subdir`.
- [x] Preserve explicit `--out-dir` override behavior (exact directory respected).
- [x] Preserve offline conversion resumability semantics (`state.json` offset + TSV append).
- [x] Keep offline manifest/schema policy unchanged (still one manifest per output dir).
- [x] Keep offline load command unchanged (`--in-dir` points at a directory containing `manifest.json`).
- [x] Update canonical config `config/bulk-merge.toml` to include `libgen.offline.layout`.
- [x] Update `docs/config.md` to document `libgen.offline.layout`.
- [x] Update `docs/cli.md` to document default kind-subdir behavior.
- [x] Check off `bulk-merge libgen convert` kind-specific cache-dir defaulting in LibGen roadmap.
- [x] Check off `libgen.offline.layout` in config roadmap.
- [x] Preserve `paths.cache_dir` / `paths.cache_policy` semantics for deriving base cache outputs.
- [x] Avoid introducing new temp files outside configured/cache-derived output directories.
- [x] Keep streaming ingest/update paths unchanged (DB-backed checkpoints only).
- [x] Keep MySQL→Postgres 1-to-1 field mapping unchanged.
- [x] Keep post-load indexing policy unchanged.
- [x] Keep COPY file streaming chunk knob unchanged.
- [x] Avoid adding new dependencies for this layout policy.
- [x] Keep tracing instrumentation on offline convert intact.
- [x] Ensure offline convert still `create_dir_all` for the resolved output dir.
- [x] Ensure error message remains clear when cache defaulting is disabled (`paths.cache_policy="never"`).
- [x] Keep config normalization behavior stable (no breaking changes for existing configs).
- [x] Keep config validation behavior stable.
- [x] Keep documentation aligned with config defaults.
- [x] Keep roadmap checkbox state aligned with repo reality.
- [x] Ensure project builds/tests after changes.
- [x] Record tranche under `docs/tranches/` for auditability.
- [x] Provide a plain (non-semver) commit message for this tranche.

