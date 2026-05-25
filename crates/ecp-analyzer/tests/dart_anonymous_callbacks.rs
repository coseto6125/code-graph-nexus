//! Calls inside an anonymous closure passed as a call argument must be
//! attached to an `<anonymous>` Function node instead of being dropped by
//! `attach_to_enclosing` (no named enclosing scope at module top-level).
//!
//! Repro: `list.forEach((x) { process(x); })` at top level produced
//! 0 callers for `process` before this change.

use ecp_analyzer::dart::parser::DartProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::LocalGraph;
use ecp_core::graph::NodeKind;
use std::path::Path;

fn parse(src: &str) -> LocalGraph {
    let p = DartProvider::new().expect("provider");
    p.parse_file(Path::new("test.dart"), src.as_bytes())
        .expect("parse")
}

fn anonymous_calls(g: &LocalGraph) -> Vec<&str> {
    g.nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Function && n.name.starts_with("<anonymous"))
        .flat_map(|n| n.calls.iter().map(String::as_str))
        .collect()
}

#[test]
fn block_closure_callback_attaches_call_to_anonymous_node() {
    // `forEach` with a block-body closure: `(x) { process(x); }`.
    // Multi-line: attach_to_enclosing is row-granular, so the inner call must
    // sit on a row the closure spans more narrowly than the enclosing function.
    let g = parse("void main() {\n  list.forEach((x) {\n    process(x);\n  });\n}");
    assert!(
        anonymous_calls(&g).contains(&"process"),
        "expected process attached to <anonymous>, nodes: {:?}",
        g.nodes
    );
}

#[test]
fn arrow_closure_direct_call_body_is_recovered() {
    // tree-sitter-dart 0.2.0 mis-parses `(x) => transform(x)` as the
    // application `((x) => transform)(x)`. receiver_types recovers the callee
    // (the closure body's leading identifier) and parser.rs emits the node
    // unconditionally for the mis-parse shape.
    let g = parse("void main() {\n  items.map((x) =>\n      transform(x));\n}");
    assert!(
        anonymous_calls(&g).contains(&"transform"),
        "expected transform recovered onto <anonymous>, nodes: {:?}",
        g.nodes
    );
}

#[test]
fn arrow_closure_method_call_body_is_recovered() {
    // `(x) => obj.method(x)` mis-parses as `((x) => obj).method(x)`; the member
    // arm already yields `method` as callee.
    let g = parse("void main() {\n  items.map((x) =>\n      obj.method(x));\n}");
    assert!(
        anonymous_calls(&g).contains(&"method"),
        "expected method recovered onto <anonymous>, nodes: {:?}",
        g.nodes
    );
}

#[test]
fn arrow_closure_binary_body_call_is_a_known_gap() {
    // `(x) => a + f(x)` mis-parses with the call fused into an additive_expr;
    // the callee is not a clean identifier so the call edge is not recovered.
    // The <anonymous> node is still emitted. Pinned so a future grammar bump
    // (or precedence-aware recovery) that surfaces the call flips this test.
    let g = parse("void main() {\n  items.map((x) =>\n      a + f(x));\n}");
    assert!(
        !anonymous_calls(&g).contains(&"f"),
        "binary-body arrow call now recovered — extend recovery / drop this pin; nodes: {:?}",
        g.nodes
    );
}

#[test]
fn top_level_closure_callback_attaches_call() {
    // Closure at module top level (no enclosing named function) — the bug
    // case where attach_to_enclosing previously had nowhere to attach.
    let g = parse("final _ = list.forEach((x) { doWork(x); });");
    assert!(
        anonymous_calls(&g).contains(&"doWork"),
        "expected doWork attached to <anonymous>, nodes: {:?}",
        g.nodes
    );
}

#[test]
fn empty_closure_emits_no_anonymous_node() {
    // A closure with no calls must not produce an <anonymous> node.
    let g = parse("void main() { items.map((x) => x * 2); }");
    assert!(
        !g.nodes
            .iter()
            .any(|n| n.kind == NodeKind::Function && n.name.starts_with("<anonymous")),
        "closure without a call must not emit a node, nodes: {:?}",
        g.nodes
    );
}

#[test]
fn named_function_arg_is_not_treated_as_anonymous_callback() {
    // Passing a named function reference — not a closure, not anonymous.
    let g = parse("void main() { list.forEach(processItem); }");
    assert!(
        !g.nodes
            .iter()
            .any(|n| n.kind == NodeKind::Function && n.name.starts_with("<anonymous")),
        "named-fn reference must not emit <anonymous>, nodes: {:?}",
        g.nodes
    );
}
