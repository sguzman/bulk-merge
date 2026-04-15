# Cache layout (`paths.cache_dir`)

`bulk-merge` writes cacheable intermediate artifacts (and some optional outputs) under
`paths.cache_dir` (default: `./.cache/bulk-merge`) when:

- a command supports caching, and
- an explicit output path is not provided, and
- `paths.cache_policy` is not `"never"`.

This directory is intended to be safe to delete when you want to reclaim space, with one caveat:
deleting it will remove resumability state for offline conversion runs that rely on `state.json`.

## LibGen offline artifacts

When running `bulk-merge libgen convert` without `--out-dir` and with the default
`libgen.offline.layout = "kind_subdir"`, artifacts are written to:

- `${paths.cache_dir}/libgen-offline/fiction/`
- `${paths.cache_dir}/libgen-offline/compact/`

Each directory contains:

- `manifest.json`: discovered table definitions and naming prefixes used during load
- `state.json`: the last processed byte offset in the dump (resumability)
- `*.tsv`: one file per MySQL table (append-only during resumable conversion)

Offline loading uses:

- `bulk-merge libgen load --in-dir <dir>`

`--in-dir` must point at a directory containing `manifest.json` and the referenced `*.tsv` files.

## Optional report output

If `output.report_path` is set, the report file location is always the explicit path you provide
(it is not automatically placed under `paths.cache_dir`).

## Cleanup expectations

- Deleting `${paths.cache_dir}/libgen-offline/<kind>/` will remove resumability state (`state.json`)
  and intermediate TSV artifacts for that dump kind.
- Deleting `${paths.cache_dir}` does not affect already-loaded PostgreSQL tables.

