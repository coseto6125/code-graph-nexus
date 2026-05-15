//! Go `type_annotation` capture coverage (Wave 2 / Task D1).
//!
//! Pins that the Go provider now populates `RawNode.type_annotation` for:
//!   * function / method parameter names (Variable nodes)
//!   * struct field names (Property nodes)
//!   * function / method return types (Function / Method nodes)
//!   * top-level `var` declarations with an explicit type (Variable nodes)
//!
//! Short declarations (`n := 1`) intentionally leave `type_annotation=None`
//! because Go's grammar exposes no `type:` field there — `gnx context` should
//! reflect that the type is inferred, not made up.
//!
//! Spec: `docs/specs/2026-05-15-language-coverage-gaps.md` Wave 2 / D1.

use graph_nexus_analyzer::go::parser::GoProvider;
use graph_nexus_core::analyzer::provider::LanguageProvider;
use graph_nexus_core::analyzer::types::RawNode;
use graph_nexus_core::graph::NodeKind;
use std::path::Path;

fn parse(src: &str) -> Vec<RawNode> {
    let provider = GoProvider::new().expect("GoProvider init");
    let graph = provider
        .parse_file(Path::new("test.go"), src.as_bytes())
        .expect("parse_file");
    graph.nodes
}

fn find<'a>(nodes: &'a [RawNode], name: &str, kind: NodeKind) -> &'a RawNode {
    nodes
        .iter()
        .find(|n| n.name == name && n.kind == kind)
        .unwrap_or_else(|| panic!("missing {kind:?} `{name}` in {nodes:#?}"))
}

#[test]
fn param_type_int() {
    let src = "package p\nfunc f(x int) {}\n";
    let nodes = parse(src);
    let x = find(&nodes, "x", NodeKind::Variable);
    assert_eq!(x.type_annotation.as_deref(), Some("int"));
}

#[test]
fn param_type_slice() {
    let src = "package p\nfunc f(s []string) {}\n";
    let nodes = parse(src);
    let s = find(&nodes, "s", NodeKind::Variable);
    assert_eq!(s.type_annotation.as_deref(), Some("[]string"));
}

#[test]
fn param_type_pointer() {
    let src = "package p\ntype User struct{}\nfunc f(u *User) {}\n";
    let nodes = parse(src);
    let u = find(&nodes, "u", NodeKind::Variable);
    assert_eq!(u.type_annotation.as_deref(), Some("*User"));
}

#[test]
fn param_type_map() {
    let src = "package p\nfunc f(m map[string]int) {}\n";
    let nodes = parse(src);
    let m = find(&nodes, "m", NodeKind::Variable);
    assert_eq!(m.type_annotation.as_deref(), Some("map[string]int"));
}

#[test]
fn field_type_basic() {
    let src = "package p\ntype S struct {\n  X int\n}\n";
    let nodes = parse(src);
    let x = find(&nodes, "X", NodeKind::Property);
    assert_eq!(x.type_annotation.as_deref(), Some("int"));
}

#[test]
fn field_type_slice() {
    let src = "package p\ntype S struct {\n  Tags []string\n}\n";
    let nodes = parse(src);
    let tags = find(&nodes, "Tags", NodeKind::Property);
    assert_eq!(tags.type_annotation.as_deref(), Some("[]string"));
}

#[test]
fn return_type_single() {
    let src = "package p\nfunc f() int { return 0 }\n";
    let nodes = parse(src);
    let f = find(&nodes, "f", NodeKind::Function);
    assert_eq!(f.type_annotation.as_deref(), Some("int"));
}

#[test]
fn return_type_multi() {
    let src = "package p\nfunc f() (int, error) { return 0, nil }\n";
    let nodes = parse(src);
    let f = find(&nodes, "f", NodeKind::Function);
    // The grammar captures the whole `(int, error)` parameter_list span.
    assert_eq!(f.type_annotation.as_deref(), Some("(int, error)"));
}

#[test]
fn var_declaration_explicit() {
    let src = "package p\nvar n int = 1\n";
    let nodes = parse(src);
    let n = find(&nodes, "n", NodeKind::Variable);
    assert_eq!(n.type_annotation.as_deref(), Some("int"));
}

#[test]
fn var_declaration_inferred_no_annotation() {
    // Short declarations have no `type:` field in the grammar — the
    // provider must NOT invent one. Inside a function body, `n := 1` is
    // a `short_var_declaration`; it should not emit a Variable node with
    // a type annotation. If a Variable `n` is emitted at all, its
    // `type_annotation` must be None.
    let src = "package p\nfunc f() { n := 1; _ = n }\n";
    let nodes = parse(src);
    if let Some(n) = nodes
        .iter()
        .find(|n| n.name == "n" && n.kind == NodeKind::Variable)
    {
        assert!(
            n.type_annotation.is_none(),
            "inferred-type `n` must have no type_annotation, got {:?}",
            n.type_annotation
        );
    }
    // Either way, the previous assert covers the contract: no fabricated type.
}
