//! TypeScript-side `path_literals` extractor regression tests.

use ecp_analyzer::typescript::parser::TypeScriptProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::RawPathLiteral;
use std::path::Path;

fn parse_path_literals(src: &str) -> Vec<RawPathLiteral> {
    let provider = TypeScriptProvider::new().expect("TypeScriptProvider::new");
    let graph = provider
        .parse_file(Path::new("test.ts"), src.as_bytes())
        .expect("parse_file");
    graph
        .path_literals
        .map(|b| b.into_vec())
        .unwrap_or_default()
}

fn find_by_value<'a>(lits: &'a [RawPathLiteral], value: &str) -> &'a RawPathLiteral {
    lits.iter()
        .find(|l| l.value == value)
        .unwrap_or_else(|| panic!("expected literal {value:?}, got: {lits:?}"))
}

#[test]
fn function_with_read_sink() {
    let src = r#"
import * as fs from "fs";
function load(): string {
    return fs.readFileSync("session_meta.json", "utf-8");
}
"#;
    let lits = parse_path_literals(src);
    let lit = find_by_value(&lits, "session_meta.json");
    assert_eq!(lit.enclosing_symbol.as_deref(), Some("load"));
    assert!(
        lit.sink_reason.starts_with("sink:read"),
        "got: {}",
        lit.sink_reason
    );
}

#[test]
fn format_string_rejected() {
    let src = r#"
function shout(): void {
    console.log("hello\nworld");
}
"#;
    let lits = parse_path_literals(src);
    assert!(!lits.iter().any(|l| l.value.contains("hello")));
}

#[test]
fn pr357_minirepro_both_literals_surface() {
    let src = r#"
import * as fs from "fs";
function reader(): string {
    return fs.readFileSync("meta.json", "utf-8");
}
function writer(d: string): void {
    fs.writeFileSync("session_meta.json", d);
}
"#;
    let lits = parse_path_literals(src);
    assert!(lits.iter().any(|l| l.value == "meta.json"));
    assert!(lits.iter().any(|l| l.value == "session_meta.json"));
}
