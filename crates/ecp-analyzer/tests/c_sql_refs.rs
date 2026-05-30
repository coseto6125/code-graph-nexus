use ecp_analyzer::c::parser::CProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = CProvider::new().expect("provider");
    let graph = provider
        .parse_file(Path::new("channel.c"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn c_select_yields_read_ref() {
    let src = "void list_channels(PGconn *conn) {\n    PQexec(conn, \"SELECT id FROM channels WHERE org_id = $1\");\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn c_insert_yields_write_ref() {
    let src = "void insert_order(PGconn *conn) {\n    PQexec(conn, \"INSERT INTO orders (id, amount) VALUES ($1, $2)\");\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "orders"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables[0].1, SqlVerb::Write);
}

#[test]
fn c_enclosing_symbol_captured() {
    let src = "void fetch_users(PGconn *conn) {\n    PQexec(conn, \"SELECT id, name FROM users ORDER BY name\");\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "users"))
        .expect("ref");
    assert_eq!(r.enclosing_symbol.as_deref(), Some("fetch_users"));
    assert_eq!(r.enclosing_owner, None);
}

#[test]
fn c_non_sql_string_not_emitted() {
    let src = "void init(void) {\n    log_message(\"Hello, world!\");\n}\n";
    let refs = parse_sql_refs(src);
    assert!(
        refs.is_empty(),
        "non-SQL string must not surface as SqlRef: {refs:?}"
    );
}
