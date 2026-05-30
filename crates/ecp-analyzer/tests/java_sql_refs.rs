use ecp_analyzer::java::parser::JavaProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = JavaProvider::new().expect("provider");
    let graph = provider
        .parse_file(Path::new("ChannelDao.java"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn java_select_yields_read_ref() {
    let src = "class ChannelDao {\n  void listChannels() {\n    jdbcTemplate.query(\"SELECT id FROM channels WHERE org_id = ?\", rowMapper);\n  }\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn java_update_yields_write_ref() {
    let src = "class ChannelDao {\n  void rename() {\n    jdbcTemplate.update(\"UPDATE channels SET slug = ? WHERE id = ?\");\n  }\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Write)]);
}

#[test]
fn java_text_block_select_yields_read_ref() {
    let src = "class ChannelDao {\n  void list() {\n    jdbcTemplate.query(\"\"\"\n      SELECT id, slug FROM channels WHERE org_id = ?\n      \"\"\", rowMapper);\n  }\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref from text block");
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn java_enclosing_symbol_and_owner_resolved() {
    let src = "class ChannelDao {\n  void listChannels() {\n    jdbcTemplate.query(\"SELECT id FROM channels WHERE org_id = ?\", rowMapper);\n  }\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert_eq!(r.enclosing_symbol.as_deref(), Some("listChannels"));
    assert_eq!(r.enclosing_owner.as_deref(), Some("ChannelDao"));
}
