-- Move raw_statement and seen_pk from src_libgen to bm_meta schema
-- Rename mysql_table to table_name in raw_statement for generality

DO $$
BEGIN
    -- Move raw_statement
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'src_libgen' AND table_name = 'raw_statement') THEN
        IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'bm_meta' AND table_name = 'raw_statement') THEN
            ALTER TABLE src_libgen.raw_statement SET SCHEMA bm_meta;
            ALTER TABLE bm_meta.raw_statement RENAME COLUMN mysql_table TO table_name;
        ELSE
            -- Already exists in target, safe to drop source
            DROP TABLE src_libgen.raw_statement;
        END IF;
    END IF;

    -- Move seen_pk
    IF EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'src_libgen' AND table_name = 'seen_pk') THEN
        IF NOT EXISTS (SELECT 1 FROM information_schema.tables WHERE table_schema = 'bm_meta' AND table_name = 'seen_pk') THEN
            ALTER TABLE src_libgen.seen_pk SET SCHEMA bm_meta;
        ELSE
            -- Already exists in target, safe to drop source
            DROP TABLE src_libgen.seen_pk;
        END IF;
    END IF;
END $$;

-- Ensure tables exist in bm_meta (for new installations)
CREATE TABLE IF NOT EXISTS bm_meta.raw_statement (
    import_run_id bigint NOT NULL REFERENCES bm_meta.import_run(id),
    byte_offset_end bigint NOT NULL,
    stmt_kind text NOT NULL, -- "create_table" | "insert_into" | "other"
    table_name text,
    sha256 bytea NOT NULL,
    payload text NOT NULL,
    inserted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (import_run_id, byte_offset_end)
);

CREATE TABLE IF NOT EXISTS bm_meta.seen_pk (
    import_run_id bigint NOT NULL REFERENCES bm_meta.import_run(id),
    table_name text NOT NULL,
    pk_column text NOT NULL,
    pk_value text NOT NULL,
    inserted_at timestamptz NOT NULL DEFAULT now(),
    PRIMARY KEY (import_run_id, table_name, pk_column, pk_value)
);
