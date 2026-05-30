use ecp_analyzer::ruby::parser::RubyProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = RubyProvider::new().expect("provider");
    let graph = provider
        .parse_file(Path::new("channel_repo.rb"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn ruby_insert_yields_write_ref() {
    let src = "def create_channel(conn)\n  conn.exec_params(\"INSERT INTO channels (slug) VALUES ($1)\", [slug])\nend\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Write)]);
}

#[test]
fn ruby_non_sql_not_emitted() {
    let src = "def log_it\n  logger.info(\"syncing channels\")\nend\n";
    assert!(parse_sql_refs(src).is_empty());
}
