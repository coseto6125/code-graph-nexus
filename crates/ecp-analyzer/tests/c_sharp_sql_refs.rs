use ecp_analyzer::c_sharp::parser::CSharpProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = CSharpProvider::new().expect("provider");
    let graph = provider
        .parse_file(Path::new("ChannelRepo.cs"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn csharp_select_yields_read_ref() {
    let src = "class ChannelRepo {\n  void ListChannels() {\n    conn.Query<Channel>(\"SELECT id FROM channels WHERE org_id = @orgId\");\n  }\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn csharp_insert_yields_write_ref() {
    let src = r#"class OrderRepo {
    void CreateOrder() {
        db.Execute("INSERT INTO orders (id, amount) VALUES (@id, @amount)");
    }
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
fn csharp_interpolated_string_not_emitted() {
    let src = r#"class Repo {
    void Query(string table) {
        db.Execute($"SELECT * FROM {table}");
    }
}
"#;
    let refs = parse_sql_refs(src);
    assert!(
        refs.is_empty(),
        "interpolated string must not surface as SqlRef: {refs:?}"
    );
}

#[test]
fn csharp_verbatim_string_sql_emitted() {
    let src = r#"class UserRepo {
    void List() {
        db.Query(@"SELECT id, name FROM users ORDER BY name");
    }
}
"#;
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "users"))
        .expect("verbatim SQL ref");
    assert_eq!(r.tables[0].1, SqlVerb::Read);
    assert_eq!(r.enclosing_symbol.as_deref(), Some("List"));
    assert_eq!(r.enclosing_owner.as_deref(), Some("UserRepo"));
}
