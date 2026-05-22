//! Swift extension dedup — `extension TypeName { ... }` must not emit a duplicate
//! Class/Struct/Enum node. Members declared inside the extension body still emit.

use ecp_analyzer::swift::parser::SwiftProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::graph::NodeKind;
use std::path::Path;

fn parse(src: &str) -> Vec<ecp_core::analyzer::types::RawNode> {
    let provider = SwiftProvider::new().expect("SwiftProvider init");
    provider
        .parse_file(Path::new("t.swift"), src.as_bytes())
        .expect("parse_file")
        .nodes
}

/// `class Foo {} extension Foo {}` — only one Class:Foo node.
#[test]
fn test_extension_does_not_duplicate_class_node() {
    let src = "class Foo {}\nextension Foo {}\n";
    let nodes = parse(src);
    let foo_classes: Vec<_> = nodes
        .iter()
        .filter(|n| n.name == "Foo" && n.kind == NodeKind::Class)
        .collect();
    assert_eq!(
        foo_classes.len(),
        1,
        "expected exactly 1 Class:Foo, got {}: {foo_classes:#?}",
        foo_classes.len()
    );
}

/// `enum AFError {} extension AFError {}` — only one Enum:AFError node.
#[test]
fn test_extension_does_not_duplicate_enum_node() {
    let src = "enum AFError: Error {}\nextension AFError {}\n";
    let nodes = parse(src);
    let hits: Vec<_> = nodes
        .iter()
        .filter(|n| n.name == "AFError" && n.kind == NodeKind::Enum)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "expected exactly 1 Enum:AFError, got {}: {hits:#?}",
        hits.len()
    );
}

/// `struct Options {} extension Options {}` — only one Struct:Options node.
#[test]
fn test_extension_does_not_duplicate_struct_node() {
    let src = "struct Options {}\nextension Options {}\n";
    let nodes = parse(src);
    let hits: Vec<_> = nodes
        .iter()
        .filter(|n| n.name == "Options" && n.kind == NodeKind::Struct)
        .collect();
    assert_eq!(
        hits.len(),
        1,
        "expected exactly 1 Struct:Options, got {}: {hits:#?}",
        hits.len()
    );
}

/// Methods declared inside an extension body still emit as Method nodes.
#[test]
fn test_extension_method_still_emits() {
    let src = "class Foo {}\nextension Foo {\n    func bar() {}\n}\n";
    let nodes = parse(src);
    let bar = nodes
        .iter()
        .find(|n| n.name == "bar")
        .expect("bar missing from extension body");
    assert_eq!(
        bar.kind,
        NodeKind::Method,
        "bar should be Method, got {bar:#?}"
    );
}

/// Properties declared inside an extension body still emit as Property nodes.
#[test]
fn test_extension_property_still_emits() {
    let src = "class Foo {}\nextension Foo {\n    var count: Int = 0\n}\n";
    let nodes = parse(src);
    let count = nodes
        .iter()
        .find(|n| n.name == "count")
        .expect("count missing from extension body");
    assert_eq!(
        count.kind,
        NodeKind::Property,
        "count should be Property, got {count:#?}"
    );
}

/// Multiple extensions to the same type in the same file — only one base node.
#[test]
fn test_multiple_extensions_same_type_one_base_node() {
    let src = "class AppDelegate {}\nextension AppDelegate {\n    func a() {}\n}\nextension AppDelegate {\n    func b() {}\n}\n";
    let nodes = parse(src);
    let class_nodes: Vec<_> = nodes
        .iter()
        .filter(|n| n.name == "AppDelegate" && n.kind == NodeKind::Class)
        .collect();
    assert_eq!(
        class_nodes.len(),
        1,
        "expected exactly 1 Class:AppDelegate, got {}: {class_nodes:#?}",
        class_nodes.len()
    );
    // Both extension methods must still emit.
    assert!(nodes.iter().any(|n| n.name == "a"), "method a missing");
    assert!(nodes.iter().any(|n| n.name == "b"), "method b missing");
}
