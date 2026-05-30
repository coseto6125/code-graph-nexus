use ecp_analyzer::go::parser::GoProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = GoProvider::new().expect("GoProvider::new");
    let graph = provider
        .parse_file(Path::new("store.go"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn go_backtick_select_yields_read_ref() {
    let src = "package store\nfunc ListChannels(db *sql.DB) {\n  db.Query(`SELECT id, slug FROM channels WHERE org_id = $1`)\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn go_double_quote_update_yields_write_ref() {
    let src = "package store\nfunc Rename(db *sql.DB) {\n  db.Exec(\"UPDATE channels SET slug = $1 WHERE id = $2\")\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Write)]);
}
