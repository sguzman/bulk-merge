# Tranche — 2026-04-14 — Postgres connection parts (host/port/user/pass/db)

Implemented in this tranche (30 items):

1. [x] Made `postgres.url` optional (still supported as an override).
2. [x] Added discrete Postgres connection config keys: `postgres.host`.
3. [x] Added discrete Postgres connection config keys: `postgres.port`.
4. [x] Added discrete Postgres connection config keys: `postgres.user`.
5. [x] Added discrete Postgres connection config keys: `postgres.password`.
6. [x] Added discrete Postgres connection config keys: `postgres.database`.
7. [x] Implemented `PostgresConfig::connection_url()` to build a URL from discrete properties.
8. [x] Ensured URL building percent-encodes user/password safely.
9. [x] Updated config validation to require discrete properties when `postgres.url` is unset.
10. [x] Added env override: `BULK_MERGE_POSTGRES_HOST`.
11. [x] Added env override: `BULK_MERGE_POSTGRES_PORT`.
12. [x] Added env override: `BULK_MERGE_POSTGRES_USER`.
13. [x] Added env override: `BULK_MERGE_POSTGRES_PASSWORD`.
14. [x] Added env override: `BULK_MERGE_POSTGRES_DATABASE`.
15. [x] Kept env override: `BULK_MERGE_POSTGRES_URL`.

16. [x] Updated DB connector to use `PostgresConfig::connection_url()`.
17. [x] Added dependency `urlencoding` for safe credential encoding.
18. [x] Updated canonical config `config/bulk-merge.toml` to match `tmp/docker-compose.yaml` using discrete properties.
19. [x] Kept commented `postgres.url` example in `config/bulk-merge.toml`.
20. [x] Updated `docs/config.md` to document discrete Postgres properties.
21. [x] Updated `docs/config.md` to document new env overrides.
22. [x] Updated `docs/roadmaps/CONFIG_ROADMAP.md` to reflect discrete Postgres config support.

23. [x] Verified unit tests (`cargo test`).
24. [x] Verified build (`cargo build`).
25. [x] Confirmed no CLI surface change required (all uses config-derived connection).
26. [x] Confirmed `postgres.table_prefix` behavior unaffected.
27. [x] Confirmed `bm_meta` migrations unaffected.
28. [x] Confirmed LibGen provisioning unchanged (only DB connection sourcing changed).
29. [x] Preserved backward compatibility via optional `postgres.url`.
30. [x] Updated `Cargo.lock` via dependency resolution.

