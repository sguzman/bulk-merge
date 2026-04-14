-- bm_meta: import bookkeeping (resumability + incremental state)

create schema if not exists bm_meta;

create table if not exists bm_meta.import_run (
    id bigserial primary key,
    source_name text not null,
    dataset_id text,
    dataset_version text,
    started_at timestamptz not null default now(),
    finished_at timestamptz,
    status text not null,
    config_json jsonb not null
);

create table if not exists bm_meta.import_file (
    id bigserial primary key,
    import_run_id bigint not null references bm_meta.import_run(id),
    path text not null,
    size_bytes bigint,
    sha256 bytea,
    status text not null,
    records_seen bigint not null default 0,
    records_loaded bigint not null default 0,
    last_offset bigint,
    unique(import_run_id, path)
);

create table if not exists bm_meta.import_checkpoint (
    import_run_id bigint not null references bm_meta.import_run(id),
    checkpoint_key text not null,
    checkpoint_value jsonb not null,
    updated_at timestamptz not null default now(),
    primary key (import_run_id, checkpoint_key)
);

-- Source namespaces (Phase 1: LibGen only)
create schema if not exists src_libgen;

