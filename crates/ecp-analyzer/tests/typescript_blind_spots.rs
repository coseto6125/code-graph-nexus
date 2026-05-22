use ecp_analyzer::typescript::parser::TypeScriptProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use std::path::Path;

fn parse_ts(src: &str) -> ecp_core::analyzer::types::LocalGraph {
    let provider = TypeScriptProvider::new().expect("TypeScriptProvider::new");
    provider
        .parse_file(Path::new("test.ts"), src.as_bytes())
        .expect("parse_file")
}

fn kinds(g: &ecp_core::analyzer::types::LocalGraph) -> Vec<&str> {
    g.blind_spots.iter().map(|b| b.kind.as_str()).collect()
}

// ── eval: always blind regardless of literal vs variable ──

#[test]
fn ts_eval_with_variable_emits_blind_spot() {
    let src = "const code = userInput; eval(code);";
    let g = parse_ts(src);
    assert!(
        kinds(&g).contains(&"ts-eval"),
        "expected ts-eval; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn ts_eval_with_string_literal_still_emits_blind_spot() {
    // eval('code') compiles a string into runtime JS; the string IS the
    // dispatch target. Unlike import()/require() where a literal resolves
    // via Imports, eval body is always opaque to the resolver.
    let src = r#"eval("var x = 1;");"#;
    let g = parse_ts(src);
    assert!(
        kinds(&g).contains(&"ts-eval"),
        "expected ts-eval (literal arg, still blind); got: {:?}",
        kinds(&g)
    );
}

// ── new Function / Function() ──

#[test]
fn ts_new_function_emits_blind_spot() {
    let src = r#"const f = new Function("a", "return a * 2;");"#;
    let g = parse_ts(src);
    assert!(
        kinds(&g).contains(&"ts-function-ctor"),
        "expected ts-function-ctor for new Function(...); got: {:?}",
        kinds(&g)
    );
}

#[test]
fn ts_function_call_without_new_emits_blind_spot() {
    let src = r#"const f = Function("return 1;");"#;
    let g = parse_ts(src);
    assert!(
        kinds(&g).contains(&"ts-function-ctor"),
        "expected ts-function-ctor for Function(...) without new; got: {:?}",
        kinds(&g)
    );
}

// ── dynamic import: literal-vs-variable check (Constraint 2) ──

#[test]
fn ts_dynamic_import_with_variable_emits_blind_spot() {
    let src = "async function load(name: string) { return await import(name); }";
    let g = parse_ts(src);
    assert!(
        kinds(&g).contains(&"ts-dynamic-import"),
        "expected ts-dynamic-import for import(<var>); got: {:?}",
        kinds(&g)
    );
}

#[test]
fn ts_dynamic_import_with_literal_skipped() {
    // import("./foo") resolves via Imports edge — must NOT emit BlindSpot.
    let src = r#"async function load() { return await import("./foo"); }"#;
    let g = parse_ts(src);
    assert!(
        !kinds(&g).contains(&"ts-dynamic-import"),
        "literal import('./foo') must NOT emit; got: {:?}",
        kinds(&g)
    );
}

// ── dynamic require ──

#[test]
fn ts_dynamic_require_with_variable_emits_blind_spot() {
    let src = "function load(name) { return require(name); }";
    let g = parse_ts(src);
    assert!(
        kinds(&g).contains(&"ts-dynamic-require"),
        "expected ts-dynamic-require for require(<var>); got: {:?}",
        kinds(&g)
    );
}

#[test]
fn ts_dynamic_require_with_literal_skipped() {
    let src = r#"const fs = require("fs");"#;
    let g = parse_ts(src);
    assert!(
        !kinds(&g).contains(&"ts-dynamic-require"),
        "literal require('fs') must NOT emit; got: {:?}",
        kinds(&g)
    );
}

// ── span convention (Constraint 3): outermost call expression ──

#[test]
fn ts_eval_span_covers_whole_call() {
    let src = "eval(payload);";
    let g = parse_ts(src);
    let bs = g
        .blind_spots
        .iter()
        .find(|b| b.kind == "ts-eval")
        .expect("ts-eval BlindSpot");
    // Span = the whole `eval(payload)` call, not just the `eval` identifier.
    // `eval(payload);` is 14 chars; identifier alone would be 4.
    let (sr, sc, er, ec) = bs.span;
    assert_eq!(sr, 0, "start row");
    assert_eq!(sc, 0, "start col");
    assert_eq!(er, 0, "end row (single-line)");
    assert!(
        ec >= 13,
        "end col should cover whole call expr (>=13); got {}",
        ec
    );
}

// ── negative: unrelated calls produce no BlindSpot ──

#[test]
fn ts_ordinary_call_produces_no_blind_spot() {
    let src = "function add(a: number, b: number) { return a + b; } add(1, 2);";
    let g = parse_ts(src);
    assert!(
        g.blind_spots.is_empty(),
        "ordinary call must not emit any BlindSpot; got: {:?}",
        g.blind_spots
    );
}
