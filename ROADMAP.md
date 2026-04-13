# bulk-merge — Implementation Roadmap

This roadmap is the source of truth for planned and completed work.

## Tranche 0 — Project Foundations

- [ ] Define and document the project goal and non-goals (in `README.md`).
- [ ] Define CLI surface area (commands, args, exit codes) in `docs/cli.md`.
- [ ] Establish crate layout (`lib.rs` + `main.rs`) and module boundaries.
- [ ] Add structured logging via `tracing` + `tracing-subscriber` (configurable).
- [ ] Add error handling strategy (`thiserror` + `anyhow` for top-level context).
- [ ] Add config loading (TOML) and validated config model.
- [ ] Add `--config` flag and env override support (documented).
- [ ] Add baseline unit tests for config parsing/validation.
- [ ] Add `clap`-based CLI parsing with help/version output.
- [ ] Add `--dry-run` support for all mutating operations.
- [ ] Add deterministic output formatting (human + JSON where applicable).
- [ ] Add `cargo fmt` + `cargo clippy` gating in CI.
- [ ] Add GitHub Actions workflow for `fmt`, `clippy`, and `test`.

## Tranche 1 — Core Capability (Bulk Merge)

- [ ] Define “bulk merge” target domain (git branches vs GitHub PRs) and data model.
- [ ] Implement input discovery (from file, stdin, or query) for merge targets.
- [ ] Implement plan/explain mode that prints intended actions.
- [ ] Implement execution engine with concurrency limits and retries (configurable).
- [ ] Implement per-target result reporting with stable error codes.
- [ ] Add integration-style tests for the execution engine (mocked interfaces).

