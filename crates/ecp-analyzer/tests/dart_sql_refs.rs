use ecp_analyzer::dart::parser::DartProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = DartProvider::new().expect("provider");
    let graph = provider
        .parse_file(Path::new("channel_repo.dart"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn dart_select_yields_read_ref() {
    let src =
        "void listChannels(db) {\n  db.query(\"SELECT id FROM channels WHERE org_id = ?\");\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn dart_insert_yields_write_ref() {
    let src = r#"void createOrder(db) {
  db.execute("INSERT INTO orders (id, amount) VALUES (?, ?)");
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
fn dart_interpolated_string_not_emitted() {
    let src = r#"void query(db, String table) {
  db.query("SELECT * FROM $table");
}
"#;
    let refs = parse_sql_refs(src);
    assert!(
        refs.is_empty(),
        "interpolated string must not surface as SqlRef: {refs:?}"
    );
}

#[test]
fn dart_enclosing_symbol_and_owner_captured() {
    let src = r#"class UserRepo {
  void listUsers(db) {
    db.query("SELECT id, name FROM users ORDER BY name");
  }
}
"#;
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "users"))
        .expect("ref");
    assert_eq!(r.tables[0].1, SqlVerb::Read);
    assert_eq!(r.enclosing_symbol.as_deref(), Some("listUsers"));
    assert_eq!(r.enclosing_owner.as_deref(), Some("UserRepo"));
}
