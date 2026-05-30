use ecp_analyzer::kotlin::parser::KotlinProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = KotlinProvider::new().expect("provider");
    let graph = provider
        .parse_file(Path::new("ChannelDao.kt"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn kotlin_select_yields_read_ref() {
    let src = "class ChannelDao {\n  fun listChannels() {\n    db.query(\"SELECT id FROM channels WHERE org_id = ?\")\n  }\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn kotlin_insert_yields_write_ref() {
    let src = r#"
class UserRepo {
    fun save() {
        db.exec("INSERT INTO users (id, name) VALUES (?, ?)")
    }
}
"#;
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "users"))
        .expect("ref");
    assert_eq!(r.tables, vec![("users".to_string(), SqlVerb::Write)]);
    assert_eq!(r.enclosing_symbol.as_deref(), Some("save"));
    assert_eq!(r.enclosing_owner.as_deref(), Some("UserRepo"));
}

#[test]
fn kotlin_multiline_sql_yields_ref() {
    let src = r#"
class OrderDao {
    fun listOrders() {
        db.query("""
            SELECT id, total
            FROM orders
            WHERE status = 'open'
        """)
    }
}
"#;
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "orders"))
        .expect("ref");
    assert_eq!(r.tables, vec![("orders".to_string(), SqlVerb::Read)]);
}

#[test]
fn kotlin_non_sql_string_not_emitted() {
    let src = r#"
class Config {
    fun load() {
        val path = "config/application.yml"
        val greeting = "hello world"
    }
}
"#;
    let refs = parse_sql_refs(src);
    assert!(refs.is_empty(), "expected no sql_refs, got: {refs:?}");
}
