use ecp_analyzer::php::parser::PhpProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{RawSqlRef, SqlVerb};
use std::path::Path;

fn parse_sql_refs(src: &str) -> Vec<RawSqlRef> {
    let provider = PhpProvider::new().expect("provider");
    let graph = provider
        .parse_file(Path::new("ChannelRepo.php"), src.as_bytes())
        .expect("parse_file");
    graph.sql_refs.map(|b| b.into_vec()).unwrap_or_default()
}

#[test]
fn php_update_yields_write_ref() {
    let src = "<?php\nfunction renameChannel($pdo) {\n  $pdo->prepare(\"UPDATE channels SET slug = :s WHERE id = :id\");\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "channels"))
        .expect("ref");
    assert!(!r.unresolved);
    assert_eq!(r.tables, vec![("channels".to_string(), SqlVerb::Write)]);
}

#[test]
fn php_select_yields_read_ref() {
    let src = "<?php\nfunction getUser($db) {\n  $db->query(\"SELECT id, name FROM users WHERE id = 1\");\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "users"))
        .expect("ref");
    assert_eq!(r.tables, vec![("users".to_string(), SqlVerb::Read)]);
}

#[test]
fn php_single_quoted_sql() {
    let src = "<?php\nfunction deleteOld($pdo) {\n  $pdo->exec('DELETE FROM sessions WHERE expired = 1');\n}\n";
    let refs = parse_sql_refs(src);
    let r = refs
        .iter()
        .find(|r| r.tables.iter().any(|(t, _)| t == "sessions"))
        .expect("ref");
    assert_eq!(r.tables, vec![("sessions".to_string(), SqlVerb::Write)]);
}

#[test]
fn php_interpolated_string_skipped() {
    // Interpolated strings are not safe to parse as SQL — must be skipped.
    let src = "<?php\nfunction search($db, $table) {\n  $db->query(\"SELECT * FROM $table\");\n}\n";
    let refs = parse_sql_refs(src);
    // interpolated double-quoted string → skipped; no refs expected
    assert!(
        refs.is_empty(),
        "interpolated SQL should be skipped, got: {refs:?}"
    );
}
