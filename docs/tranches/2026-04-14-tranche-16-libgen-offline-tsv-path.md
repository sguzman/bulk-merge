# 2026-04-14 — Tranche 16: LibGen offline TSV path

Implemented items (30):

- [x] Add `execution.copy.file_send_chunk_bytes` config knob for streaming intermediate files into `COPY FROM STDIN`.
- [x] Add `[execution.copy]` section to canonical `config/bulk-merge.toml`.
- [x] Enable `tokio` `fs` + `io-util` features for async file streaming into COPY.
- [x] Add `Db::copy_in_tsv_file(...)` to stream `.tsv` bytes into Postgres COPY without loading whole files in memory.
- [x] Keep COPY format consistent between streaming ingest and offline TSV load (CSV format, tab delimiter, `\\N` NULL, quoted fields).
- [x] Add `libgen.offline.out_dir_default` control-pane knob.
- [x] Add `[libgen.offline]` section to canonical `config/bulk-merge.toml`.
- [x] Implement resumable offline conversion state (`state.json` with byte offset).
- [x] Implement offline conversion manifest (`manifest.json` with table defs and naming prefixes).
- [x] Ensure offline conversion writes per-MySQL-table `*.tsv` files (append mode for resumability).
- [x] Add periodic percent-based progress logs to offline conversion (based on dump file byte offsets).
- [x] Add periodic progress logs to offline load (per-table milestones + file byte hints).
- [x] Wire offline conversion into CLI as `bulk-merge libgen convert`.
- [x] Default `bulk-merge libgen convert --out-dir` from `libgen.offline.out_dir_default` when not provided.
- [x] Wire offline load into CLI as `bulk-merge libgen load`.
- [x] Ensure offline load runs migrations before loading TSV artifacts.
- [x] Ensure offline load registers an `import_run` in `bm_meta` for traceability.
- [x] Ensure offline load marks the `import_run` as succeeded on completion.
- [x] Ensure offline load updates `bm_meta.dataset_state` using the manifest kind string.
- [x] Add docs for offline commands in `docs/cli.md`.
- [x] Document offline config knobs in `docs/config.md`.
- [x] Document that raw landing table name is currently migration-defined (not configurable yet).
- [x] Mark LibGen offline roadmap items completed in `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`.
- [x] Update config roadmap with offline and COPY-file streaming knobs.
- [x] Split raw-table configurability roadmap item into “document fixed name” vs “make configurable later”.
- [x] Add structured `tracing` spans around offline convert/load entrypoints.
- [x] Preserve 1-to-1 column mapping (still `text`) for offline load path.
- [x] Preserve “indexes after bulk insert” policy in offline load path.
- [x] Avoid full-file buffering in offline conversion (statement streaming + per-table buffered writers).
- [x] Avoid full-file buffering in offline load (chunked streaming into COPY).

