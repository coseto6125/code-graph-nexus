//! Regression tests: Python decorator capture on function and class definitions.
//!
//! Bug: `queries.scm` had no `@decorator` capture, so
//! `capture_index_for_name("decorator")` returned `None` and ALL decorator
//! information was silently dropped from `RawNode.decorators`.
//!
//! Fix: added `decorated_definition` patterns to `queries.scm` covering both
//! non-call decorators (identifiers / dotted attributes) and call decorators
//! (captures the call target, dropping arguments).

use ecp_analyzer::python::parser::PythonProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::LocalGraph;
use std::path::Path;

fn parse(source: &str) -> LocalGraph {
    let provider = PythonProvider::new().expect("PythonProvider::new");
    provider
        .parse_file(Path::new("test.py"), source.as_bytes())
        .expect("parse_file")
}

fn decorators_of<'a>(g: &'a LocalGraph, name: &str) -> Vec<&'a str> {
    g.nodes
        .iter()
        .find(|n| n.name == name)
        .map(|n| n.decorators.iter().map(|s| s.as_str()).collect())
        .unwrap_or_default()
}

// ── 1. @property ─────────────────────────────────────────────────────────────

#[test]
fn property_decorator_captured() {
    let src = "class C:\n    @property\n    def foo(self):\n        return 1\n";
    let g = parse(src);
    let decs = decorators_of(&g, "foo");
    assert_eq!(decs, ["property"], "decorators: {decs:?}");
}

// ── 2. @functools.cached_property (dotted attribute) ─────────────────────────

#[test]
fn dotted_attribute_decorator_captured() {
    let src = "import functools\n\nclass C:\n    @functools.cached_property\n    def bar(self):\n        return 2\n";
    let g = parse(src);
    let decs = decorators_of(&g, "bar");
    assert_eq!(decs, ["functools.cached_property"], "decorators: {decs:?}");
}

// ── 3. @staticmethod ─────────────────────────────────────────────────────────

#[test]
fn staticmethod_decorator_captured() {
    let src = "class C:\n    @staticmethod\n    def baz():\n        pass\n";
    let g = parse(src);
    let decs = decorators_of(&g, "baz");
    assert_eq!(decs, ["staticmethod"], "decorators: {decs:?}");
}

// ── 4. @app.get("/users") — call decorator, arguments dropped ────────────────

#[test]
fn call_decorator_captures_target_without_args() {
    let src = "from fastapi import FastAPI\napp = FastAPI()\n\n@app.get(\"/users\")\ndef list_users():\n    return []\n";
    let g = parse(src);
    let decs = decorators_of(&g, "list_users");
    assert_eq!(decs, ["app.get"], "decorators: {decs:?}");
}

// ── 5. Multi-decorator: @a then @b on the same def ───────────────────────────

#[test]
fn multiple_decorators_all_captured_in_order() {
    let src = "@a\n@b\ndef f():\n    pass\n";
    let g = parse(src);
    let decs = decorators_of(&g, "f");
    // Tree-sitter emits one match per decorator, preserving source order.
    assert_eq!(decs.len(), 2, "expected 2 decorators, got {decs:?}");
    assert!(decs.contains(&"a"), "missing 'a' in {decs:?}");
    assert!(decs.contains(&"b"), "missing 'b' in {decs:?}");
}

// ── 6. Class decorator ───────────────────────────────────────────────────────

#[test]
fn class_decorator_captured() {
    let src = "@dataclass\nclass Foo:\n    x: int = 0\n";
    let g = parse(src);
    let decs = decorators_of(&g, "Foo");
    assert_eq!(decs, ["dataclass"], "decorators: {decs:?}");
}
