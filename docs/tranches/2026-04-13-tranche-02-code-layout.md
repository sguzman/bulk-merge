# Tranche — 2026-04-13 — Code layout refactor (module growth)

Implemented in this tranche:

1. [x] Split CLI into `src/cli/` with `args.rs` and `commands/` modules.
2. [x] Split command implementations into `src/cli/commands/init_db.rs` and `src/cli/commands/libgen.rs`.
3. [x] Split config into `src/config/` with `model.rs` re-exported by `mod.rs`.
4. [x] Split DB into `src/db/` with `meta.rs` re-exported by `mod.rs`.
5. [x] Moved tracing init into `src/observability/tracing.rs` (re-exported by `src/observability/mod.rs`).
6. [x] Added `src/libgen/mod.rs` as the future home for LibGen parsing/ingestion logic.
7. [x] Kept `src/main.rs` minimal: arg parse → config load → tracing init → command dispatch.

Build verification:

- [x] `cargo build`

