# bulk-merge

`bulk-merge` is a Rust CLI for ingesting large bibliographic metadata dumps into PostgreSQL as usable queryable tables.

## Intent

Move bulk metadata loading out of ad hoc import scripts into a reproducible, observable ingestion tool with migrations, progress reporting, and test coverage.

## Ambition

Given the docs, migrations, and test scaffolding, the project appears intended to become a dependable ingestion layer for very large reference datasets rather than a one-shot ETL throwaway.

## Current Status

The repository already has a CLI entrypoint, migrations, docs, tests, and progress/output modules. The existing README also frames the work as an intentionally scoped early phase.

## Core Capabilities Or Focus Areas

- CLI-driven bulk ingest workflow.
- PostgreSQL migration support.
- Progress and output/reporting helpers.
- Test coverage around the import pipeline.
- Configurable runtime behavior through project config.

## Project Layout

- `config/`: checked-in runtime configuration and configuration examples.
- `docs/`: project documentation, reference material, and roadmap notes.
- `migrations/`: database schema migrations.
- `scratch/`: working notes or experimental assets that support ongoing development.
- `src/`: Rust source for the main crate or application entrypoint.
- `tests/`: automated tests, fixtures, or parity scenarios.
- `Cargo.toml`: crate or workspace manifest and the first place to check for package structure.

## Setup And Requirements

- Rust toolchain.
- PostgreSQL database reachable from the machine running the importer.
- Input metadata dumps in the format expected by the current phase.

## Build / Run / Test Commands

```bash
cargo build
cargo test
cargo run -- --help
```

## Notes, Limitations, Or Known Gaps

- The existing docs describe the current implementation as phase-based, so not every long-term ingest feature is expected to exist yet.
- Database schema and input-shape assumptions are central to the workflow.

## Next Steps Or Roadmap Hints

- Broaden import coverage while keeping migration and test discipline intact.
- Document the stable ingest contract once the current phase boundaries settle.
