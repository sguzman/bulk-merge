# 2026-04-15 — Tranche 33: Real LibGen compact offline run + parser/load fixes

Roadmap items targeted (verbatim):

- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: Parser correctness: handle doubled-quote escaping (`''`) consistently in both statement splitting and value parsing (including across buffer boundaries).
- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: Sanitize NUL bytes during offline load (Postgres `text` cannot contain NUL; configurable replacement).
- `docs/roadmaps/CONFIG_ROADMAP.md`: `libgen.dump.sanitize_nul_bytes` and `libgen.dump.nul_replacement` (Postgres text compatibility).

Run performed:

- `bulk-merge init-db`
- `bulk-merge libgen convert --kind compact --dump /drive/books/metadata/libgen/torrent/libgen_compact.sql --out-dir .cache/bulk-merge/manual-run-20260415-compact`
- `bulk-merge libgen load --in-dir .cache/bulk-merge/manual-run-20260415-compact --dataset-id libgen-compact-offline --dataset-version 2026-04-15`
- `bulk-merge libgen validate --kind compact --mysql-table updated`

Implemented items (roadmap-derived):

- [x] Fix `StatementReader` scanning to be resumable across buffer reads (avoid re-scanning with mutated quote state).
- [x] Fix statement splitting to respect doubled-quote escaping (`''`) even when escape spans buffer boundaries.
- [x] Fix value parsing to support doubled-quote escaping (`''`) within string literals.
- [x] Add config knobs for NUL sanitization (`libgen.dump.sanitize_nul_bytes`, `libgen.dump.nul_replacement`).
- [x] Sanitize NUL bytes when streaming TSV files into Postgres `COPY FROM STDIN` during offline load.
- [x] Fix `select 1` type mismatches by casting to `bigint` for SQLx compatibility.
- [x] Verify real compact dump conversion completes with percent progress logs.
- [x] Verify real compact dump offline load completes (staging+swap) and indexes are created post-load.
- [x] Verify `libgen validate` passes on `compact_updated` with non-zero row count.
- [x] Check off the targeted roadmap items as completed.

