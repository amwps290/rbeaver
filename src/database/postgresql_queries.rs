/// PostgreSQL metadata queries for database object categorization
///
/// This module contains optimized SQL queries to fetch metadata for different
/// PostgreSQL object types from system catalogs (information_schema and pg_catalog).

/// Query to get all schemas including system schemas
pub const GET_ALL_SCHEMAS_QUERY: &str = r#"
SELECT 
    schema_name,
    schema_owner
FROM information_schema.schemata 
ORDER BY 
    CASE 
        WHEN schema_name IN ('information_schema', 'pg_catalog', 'pg_toast') THEN 1
        ELSE 0
    END,
    schema_name
"#;

/// Query to get user schemas (excluding system schemas)
pub const GET_USER_SCHEMAS_QUERY: &str = r#"
SELECT 
    schema_name,
    schema_owner
FROM information_schema.schemata 
WHERE schema_name NOT IN ('information_schema', 'pg_catalog', 'pg_toast')
ORDER BY schema_name
"#;

/// Query to get views in a specific schema
pub const GET_VIEWS_QUERY: &str = r#"
SELECT 
    v.table_name as name,
    v.table_schema as schema,
    CASE 
        WHEN c.relkind = 'm' THEN 'MATERIALIZED VIEW'
        ELSE 'VIEW'
    END as view_type,
    v.view_definition as definition,
    COALESCE(obj_description(c.oid), '') as comment,
    pg_get_userbyid(c.relowner) as owner,
    v.is_updatable = 'YES' as is_updatable
FROM information_schema.views v
LEFT JOIN pg_class c ON c.relname = v.table_name
LEFT JOIN pg_namespace n ON n.oid = c.relnamespace AND n.nspname = v.table_schema
WHERE v.table_schema = $1
UNION ALL
SELECT 
    c.relname as name,
    n.nspname as schema,
    'MATERIALIZED VIEW' as view_type,
    pg_get_viewdef(c.oid) as definition,
    COALESCE(obj_description(c.oid), '') as comment,
    pg_get_userbyid(c.relowner) as owner,
    false as is_updatable
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
WHERE c.relkind = 'm' AND n.nspname = $1
ORDER BY name
"#;

/// Query to get functions and procedures in a specific schema
pub const GET_FUNCTIONS_QUERY: &str = r#"
SELECT 
    p.proname as name,
    n.nspname as schema,
    CASE 
        WHEN p.prokind = 'f' THEN 'FUNCTION'
        WHEN p.prokind = 'p' THEN 'PROCEDURE'
        WHEN p.prokind = 'a' THEN 'AGGREGATE'
        WHEN p.prokind = 'w' THEN 'WINDOW'
        ELSE 'FUNCTION'
    END as function_type,
    pg_get_function_result(p.oid) as return_type,
    pg_get_function_arguments(p.oid) as arguments,
    l.lanname as language,
    CASE 
        WHEN p.prosrc IS NOT NULL AND p.prosrc != '' THEN p.prosrc
        ELSE pg_get_functiondef(p.oid)
    END as definition,
    COALESCE(obj_description(p.oid), '') as comment,
    pg_get_userbyid(p.proowner) as owner
FROM pg_proc p
JOIN pg_namespace n ON n.oid = p.pronamespace
JOIN pg_language l ON l.oid = p.prolang
WHERE n.nspname = $1
    AND p.prokind IN ('f', 'p', 'a', 'w')
ORDER BY p.proname
"#;

/// Query to get triggers in a specific schema
pub const GET_TRIGGERS_QUERY: &str = r#"
SELECT 
    t.tgname as name,
    n.nspname as schema,
    c.relname as table_name,
    CASE 
        WHEN t.tgtype & 1 = 1 THEN 'ROW'
        ELSE 'STATEMENT'
    END as trigger_type,
    array_to_string(
        ARRAY[
            CASE WHEN t.tgtype & 4 = 4 THEN 'INSERT' END,
            CASE WHEN t.tgtype & 8 = 8 THEN 'DELETE' END,
            CASE WHEN t.tgtype & 16 = 16 THEN 'UPDATE' END,
            CASE WHEN t.tgtype & 32 = 32 THEN 'TRUNCATE' END
        ]::text[], 
        ','
    ) as events,
    CASE 
        WHEN t.tgtype & 2 = 2 THEN 'BEFORE'
        WHEN t.tgtype & 64 = 64 THEN 'INSTEAD_OF'
        ELSE 'AFTER'
    END as timing,
    p.proname as function_name,
    fn.nspname as function_schema,
    pg_get_triggerdef(t.oid) as condition,
    COALESCE(obj_description(t.oid), '') as comment
FROM pg_trigger t
JOIN pg_class c ON c.oid = t.tgrelid
JOIN pg_namespace n ON n.oid = c.relnamespace
JOIN pg_proc p ON p.oid = t.tgfoid
JOIN pg_namespace fn ON fn.oid = p.pronamespace
WHERE n.nspname = $1
    AND NOT t.tgisinternal
ORDER BY c.relname, t.tgname
"#;

/// Query to get sequences in a specific schema
pub const GET_SEQUENCES_QUERY: &str = r#"
SELECT 
    c.relname as name,
    n.nspname as schema,
    format_type(s.seqtypid, NULL) as data_type,
    s.seqstart as start_value,
    s.seqmin as min_value,
    s.seqmax as max_value,
    s.seqincrement as increment,
    s.seqcycle as cycle,
    s.seqcache as cache_size,
    CASE 
        WHEN pg_sequence_last_value(c.oid) IS NOT NULL 
        THEN pg_sequence_last_value(c.oid)
        ELSE NULL
    END as last_value,
    dep_c.relname as owner_table,
    dep_a.attname as owner_column,
    COALESCE(obj_description(c.oid), '') as comment
FROM pg_class c
JOIN pg_namespace n ON n.oid = c.relnamespace
JOIN pg_sequence s ON s.seqrelid = c.oid
LEFT JOIN pg_depend d ON d.objid = c.oid AND d.deptype = 'a'
LEFT JOIN pg_class dep_c ON dep_c.oid = d.refobjid
LEFT JOIN pg_attribute dep_a ON dep_a.attrelid = d.refobjid AND dep_a.attnum = d.refobjsubid
WHERE c.relkind = 'S' AND n.nspname = $1
ORDER BY c.relname
"#;

/// Query to get indexes in a specific schema
pub const GET_INDEXES_QUERY: &str = r#"
SELECT 
    i.relname as name,
    n.nspname as schema,
    t.relname as table_name,
    am.amname as index_type,
    ix.indisunique as is_unique,
    ix.indisprimary as is_primary,
    ix.indpred IS NOT NULL as is_partial,
    pg_get_expr(ix.indpred, ix.indrelid) as condition,
    pg_size_pretty(pg_relation_size(i.oid)) as size,
    COALESCE(obj_description(i.oid), '') as comment,
    array_to_string(
        ARRAY(
            SELECT a.attname
            FROM pg_attribute a
            WHERE a.attrelid = t.oid
                AND a.attnum = ANY(ix.indkey)
            ORDER BY array_position(ix.indkey, a.attnum)
        ),
        ','
    ) as columns
FROM pg_class i
JOIN pg_namespace n ON n.oid = i.relnamespace
JOIN pg_index ix ON ix.indexrelid = i.oid
JOIN pg_class t ON t.oid = ix.indrelid
JOIN pg_am am ON am.oid = i.relam
WHERE i.relkind = 'i' AND n.nspname = $1
ORDER BY t.relname, i.relname
"#;

/// Query to get object counts for a specific schema
pub const GET_OBJECT_COUNTS_QUERY: &str = r#"
SELECT 
    COUNT(CASE WHEN c.relkind = 'r' THEN 1 END) as tables,
    COUNT(CASE WHEN c.relkind = 'v' THEN 1 END) as views,
    COUNT(CASE WHEN c.relkind = 'm' THEN 1 END) as materialized_views,
    COUNT(CASE WHEN p.prokind = 'f' THEN 1 END) as functions,
    COUNT(CASE WHEN p.prokind = 'p' THEN 1 END) as procedures,
    COUNT(CASE WHEN t.tgname IS NOT NULL AND NOT t.tgisinternal THEN 1 END) as triggers,
    COUNT(CASE WHEN c.relkind = 'S' THEN 1 END) as sequences,
    COUNT(CASE WHEN c.relkind = 'i' THEN 1 END) as indexes
FROM pg_namespace n
LEFT JOIN pg_class c ON c.relnamespace = n.oid
LEFT JOIN pg_proc p ON p.pronamespace = n.oid
LEFT JOIN pg_trigger t ON t.tgrelid = c.oid
WHERE n.nspname = $1
"#;

/// Query to get database-wide object counts
pub const GET_DATABASE_OBJECT_COUNTS_QUERY: &str = r#"
SELECT 
    COUNT(DISTINCT n.nspname) as schemas,
    COUNT(DISTINCT CASE WHEN n.nspname NOT IN ('information_schema', 'pg_catalog', 'pg_toast') THEN n.nspname END) as user_schemas,
    COUNT(DISTINCT CASE WHEN n.nspname IN ('information_schema', 'pg_catalog', 'pg_toast') THEN n.nspname END) as system_schemas,
    COUNT(CASE WHEN c.relkind = 'r' THEN 1 END) as total_tables,
    COUNT(CASE WHEN c.relkind = 'v' THEN 1 END) as total_views,
    COUNT(CASE WHEN c.relkind = 'm' THEN 1 END) as total_materialized_views,
    COUNT(CASE WHEN p.prokind = 'f' THEN 1 END) as total_functions,
    COUNT(CASE WHEN p.prokind = 'p' THEN 1 END) as total_procedures,
    COUNT(CASE WHEN t.tgname IS NOT NULL AND NOT t.tgisinternal THEN 1 END) as total_triggers,
    COUNT(CASE WHEN c.relkind = 'S' THEN 1 END) as total_sequences,
    COUNT(CASE WHEN c.relkind = 'i' THEN 1 END) as total_indexes
FROM pg_namespace n
LEFT JOIN pg_class c ON c.relnamespace = n.oid
LEFT JOIN pg_proc p ON p.pronamespace = n.oid
LEFT JOIN pg_trigger t ON t.tgrelid = c.oid
"#;
