use ecp_analyzer::python::parser::PythonProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = PythonProvider::new().expect("PythonProvider::new");
    let graph = provider
        .parse_file(Path::new("api.py"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn select_string_yields_read_sql_ref_to_channels() {
    let src = "def list_channels(pool):\n    return pool.fetch(\"SELECT id, slug FROM channels WHERE org_id = $1\")\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("expected a sql_ref referencing channels");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Read)]);
    assert_eq!(r.enclosing_symbol.as_deref(), Some("list_channels"));
}

#[test]
fn update_string_yields_write_sql_ref() {
    let src =
        "def rename(pool):\n    pool.execute(\"UPDATE channels SET slug = $1 WHERE id = $2\")\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Write)]);
}

#[test]
fn non_sql_string_yields_no_sql_ref() {
    let src = "def log_it():\n    logger.info(\"syncing channels for org\")\n";
    let refs = parse_sql_refs(src);
    assert!(
        refs.is_empty(),
        "non-SQL string must not produce a sql_ref: {refs:?}"
    );
}
