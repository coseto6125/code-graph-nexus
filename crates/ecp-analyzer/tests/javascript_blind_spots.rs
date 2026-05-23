use ecp_analyzer::javascript::parser::JavaScriptProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use std::path::Path;

fn parse_js(src: &str) -> ecp_core::analyzer::types::LocalGraph {
    let provider = JavaScriptProvider::new().expect("JavaScriptProvider::new");
    provider
        .parse_file(Path::new("test.js"), src.as_bytes())
        .expect("parse_file")
}

fn kinds(g: &ecp_core::analyzer::types::LocalGraph) -> Vec<&str> {
    g.blind_spots.iter().map(|b| b.kind.as_str()).collect()
}

#[test]
fn js_eval_with_variable_emits_blind_spot() {
    let src = "var code = userInput; eval(code);";
    let g = parse_js(src);
    assert!(
        kinds(&g).contains(&"js-eval"),
        "expected js-eval; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn js_eval_with_string_literal_still_emits_blind_spot() {
    let src = r#"eval("var x = 1;");"#;
    let g = parse_js(src);
    assert!(
        kinds(&g).contains(&"js-eval"),
        "expected js-eval (literal arg, still blind); got: {:?}",
        kinds(&g)
    );
}

#[test]
fn js_new_function_emits_blind_spot() {
    let src = r#"var f = new Function("a", "return a * 2;");"#;
    let g = parse_js(src);
    assert!(
        kinds(&g).contains(&"js-function-ctor"),
        "expected js-function-ctor for new Function(...); got: {:?}",
        kinds(&g)
    );
}

#[test]
fn js_function_call_without_new_emits_blind_spot() {
    let src = r#"var f = Function("return 1;");"#;
    let g = parse_js(src);
    assert!(
        kinds(&g).contains(&"js-function-ctor"),
        "expected js-function-ctor for Function(...) without new; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn js_dynamic_import_with_variable_emits_blind_spot() {
    let src = "async function load(name) { return await import(name); }";
    let g = parse_js(src);
    assert!(
        kinds(&g).contains(&"js-dynamic-import"),
        "expected js-dynamic-import for import(<var>); got: {:?}",
        kinds(&g)
    );
}

#[test]
fn js_dynamic_import_with_literal_skipped() {
    let src = r#"async function load() { return await import("./foo"); }"#;
    let g = parse_js(src);
    assert!(
        !kinds(&g).contains(&"js-dynamic-import"),
        "literal import('./foo') must NOT emit; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn js_dynamic_require_with_variable_emits_blind_spot() {
    let src = "function load(name) { return require(name); }";
    let g = parse_js(src);
    assert!(
        kinds(&g).contains(&"js-dynamic-require"),
        "expected js-dynamic-require for require(<var>); got: {:?}",
        kinds(&g)
    );
}

#[test]
fn js_dynamic_require_with_literal_skipped() {
    let src = r#"var fs = require("fs");"#;
    let g = parse_js(src);
    assert!(
        !kinds(&g).contains(&"js-dynamic-require"),
        "literal require('fs') must NOT emit; got: {:?}",
        kinds(&g)
    );
}

#[test]
fn js_eval_span_covers_whole_call() {
    let src = "eval(payload);";
    let g = parse_js(src);
    let bs = g
        .blind_spots
        .iter()
        .find(|b| b.kind == "js-eval")
        .expect("js-eval BlindSpot");
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

#[test]
fn js_ordinary_call_produces_no_blind_spot() {
    let src = "function add(a, b) { return a + b; } add(1, 2);";
    let g = parse_js(src);
    assert!(
        g.blind_spots.is_empty(),
        "ordinary call must not emit any BlindSpot; got: {:?}",
        g.blind_spots
    );
}
