use ecp_analyzer::swift::parser::SwiftProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = SwiftProvider::new().expect("SwiftProvider::new");
    let graph = provider
        .parse_file(Path::new("ChannelRepo.swift"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn swift_select_yields_read_ref() {
    let src = "func listChannels(db: Database) {\n    db.query(\"SELECT id FROM channels WHERE org_id = ?\")\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}

#[test]
fn swift_update_yields_write_ref() {
    let src = "func renameChannel(db: Database) {\n    db.execute(\"UPDATE channels SET name = ? WHERE id = ?\")\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Write)]);
}
