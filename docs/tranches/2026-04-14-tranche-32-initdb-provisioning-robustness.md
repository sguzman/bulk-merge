# 2026-04-14 — Tranche 32: init-db provisioning robustness

Roadmap items targeted (verbatim):

- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: `libgen.init.dumps.*` treat empty strings as unset during config normalization.
- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: When `libgen.init.provision_tables=true`, config validation requires at least one of `libgen.init.dumps.fiction` or `libgen.init.dumps.compact` is set.
- `docs/roadmaps/LIBGEN_INGESTION_ROADMAP.md`: Document init-db provisioning failure modes and config expectations (no manual QA; just doc).
- `docs/roadmaps/CONFIG_ROADMAP.md`: Normalize empty `libgen.init.dumps.*` as unset and validate provision_tables requires at least one dump path.

Implemented items (roadmap-derived):

- [x] Normalize empty `libgen.init.dumps.*` strings to `None`.
- [x] Add config validation preventing `libgen.init.provision_tables=true` with no dump paths configured.
- [x] Document the validation and empty-string normalization in `docs/config.md`.
- [x] Check off the targeted roadmap items.
- [x] Verify `cargo test` passes after changes.

