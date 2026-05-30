use ecp_analyzer::rust::parser::RustProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = RustProvider::new().expect("RustProvider::new");
    let graph = provider
        .parse_file(Path::new("store.rs"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn rust_query_string_yields_read_ref() {
    let src = "fn list_channels(pool: &Pool) {\n    sqlx::query(\"SELECT id, slug FROM channels WHERE org_id = $1\");\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
}
