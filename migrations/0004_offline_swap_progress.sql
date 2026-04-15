-- Offline load resumability: swap progress tracking (staging schema → live schema)

create table if not exists bm_meta.offline_swap_progress (
    import_run_id bigint not null references bm_meta.import_run(id),
    schema_live text not null,
    schema_staging text not null,
    table_name text not null,
    stage text not null, -- "staged" | "swapped"
    updated_at timestamptz not null default now(),
    primary key (import_run_id, table_name)
);

