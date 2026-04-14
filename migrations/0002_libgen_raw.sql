-- LibGen raw landing tables (provenance-preserving reprocessing)

create schema if not exists src_libgen;

create table if not exists src_libgen.raw_statement (
    import_run_id bigint not null references bm_meta.import_run(id),
    byte_offset_end bigint not null,
    stmt_kind text not null, -- "create_table" | "insert_into" | "other"
    mysql_table text,
    sha256 bytea not null,
    payload text not null,
    inserted_at timestamptz not null default now(),
    primary key (import_run_id, byte_offset_end)
);

create index if not exists idx_raw_statement_mysql_table
    on src_libgen.raw_statement (mysql_table);

