-- Dataset-level state tracking (incremental updates)

create schema if not exists bm_meta;

create table if not exists bm_meta.dataset_state (
    source_name text not null,
    dataset_id text not null,
    kind text not null,
    last_succeeded_import_run_id bigint references bm_meta.import_run(id),
    last_dataset_version text,
    updated_at timestamptz not null default now(),
    primary key (source_name, dataset_id, kind)
);

-- Optional delete-handling support for LibGen updates (Phase 1).
-- This is intentionally narrow: it supports single-column primary keys (typically `ID`).

create schema if not exists src_libgen;

create table if not exists src_libgen.seen_pk (
    import_run_id bigint not null references bm_meta.import_run(id),
    table_name text not null,
    pk_column text not null,
    pk_value text not null,
    inserted_at timestamptz not null default now(),
    primary key (import_run_id, table_name, pk_column, pk_value)
);

create index if not exists idx_seen_pk_table
    on src_libgen.seen_pk (table_name);

