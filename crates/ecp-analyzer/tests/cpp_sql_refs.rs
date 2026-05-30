use ecp_analyzer::cpp::parser::CppProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = CppProvider::new().expect("provider");
    let graph = provider
        .parse_file(Path::new("channel.cpp"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn cpp_select_yields_read_ref() {
    let src = "void listChannels(Conn& c) {\n    c.exec(\"SELECT id FROM channels WHERE org_id = $1\");\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn cpp_insert_yields_write_ref() {
    let src = r#"void createOrder(Db& db) {
    db.exec("INSERT INTO orders (id, amount) VALUES ($1, $2)");
}
"#;
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "orders"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables[0].1, SqlVerb::Write);
}

#[test]
fn cpp_raw_string_literal_sql_emitted() {
    let src = r#"void listUsers(Db& db) {
    db.exec(R"(SELECT id, name FROM users ORDER BY name)");
}
"#;
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "users"))
        .expect("raw string SQL ref");
    assert_eq!(r.tables[0].1, SqlVerb::Read);
    assert_eq!(r.enclosing_symbol.as_deref(), Some("listUsers"));
}

#[test]
fn cpp_enclosing_symbol_and_owner_captured() {
    let src = r#"class UserRepo {
    void list() {
        db.exec("SELECT id FROM users");
    }
};
"#;
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "users"))
        .expect("ref");
    assert_eq!(r.enclosing_symbol.as_deref(), Some("list"));
    assert_eq!(r.enclosing_owner.as_deref(), Some("UserRepo"));
}
