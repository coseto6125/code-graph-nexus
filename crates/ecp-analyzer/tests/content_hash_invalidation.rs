//! T7-2: Per-symbol content_hash invalidation on body change.
//!
//! Parses two near-identical sources that differ by a single comment INSIDE a
//! function body. The function node's `content_hash` MUST differ between the
//! two parses — a doc-comment edit IS a content change for our purposes. The
//! incremental skip (T7-4/5/6) fires only when bytes match exactly.

use std::path::Path;

use ecp_core::analyzer::provider::LanguageProvider;

fn hash_of(provider: &dyn LanguageProvider, path: &str, src: &str, name: &str) -> u64 {
    let lg = provider
        .parse_file(Path::new(path), src.as_bytes())
        .unwrap_or_else(|e| panic!("parse_file({path}) failed: {e}"));
    lg.nodes
        .iter()
        .find(|n| n.name == name)
        .unwrap_or_else(|| panic!("symbol `{name}` not found in {path}"))
        .content_hash
}

fn assert_invalidated(
    provider: &dyn LanguageProvider,
    path: &str,
    src_original: &str,
    src_modified: &str,
    symbol_name: &str,
) {
    let h1 = hash_of(provider, path, src_original, symbol_name);
    let h2 = hash_of(provider, path, src_modified, symbol_name);
    assert_ne!(
        h1, h2,
        "{path}: `{symbol_name}` content_hash should differ when body comment is added\n\
         original={h1:#018x}  modified={h2:#018x}"
    );
}

// ── TypeScript ─────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_typescript() {
    use ecp_analyzer::typescript::parser::TypeScriptProvider;
    let p = TypeScriptProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.ts",
        "function compute(x: number): number {\n  return x * 2;\n}\n",
        "function compute(x: number): number {\n  // multiply by two\n  return x * 2;\n}\n",
        "compute",
    );
}

// ── JavaScript ─────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_javascript() {
    use ecp_analyzer::javascript::parser::JavaScriptProvider;
    let p = JavaScriptProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.js",
        "function compute(x) { return x * 2; }\n",
        "function compute(x) {\n  // double it\n  return x * 2;\n}\n",
        "compute",
    );
}

// ── Python ─────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_python() {
    use ecp_analyzer::python::parser::PythonProvider;
    let p = PythonProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.py",
        "def compute(x):\n    return x * 2\n",
        "def compute(x):\n    # multiply by two\n    return x * 2\n",
        "compute",
    );
}

// ── Java ───────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_java() {
    use ecp_analyzer::java::parser::JavaProvider;
    let p = JavaProvider::new().unwrap();
    assert_invalidated(
        &p,
        "T.java",
        "public class T {\n    public int compute(int x) { return x * 2; }\n}\n",
        "public class T {\n    public int compute(int x) {\n        // double it\n        return x * 2;\n    }\n}\n",
        "compute",
    );
}

// ── Kotlin ─────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_kotlin() {
    use ecp_analyzer::kotlin::parser::KotlinProvider;
    let p = KotlinProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.kt",
        "fun compute(x: Int): Int { return x * 2 }\n",
        "fun compute(x: Int): Int {\n    // double it\n    return x * 2\n}\n",
        "compute",
    );
}

// ── C# ─────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_csharp() {
    use ecp_analyzer::c_sharp::parser::CSharpProvider;
    let p = CSharpProvider::new().unwrap();
    assert_invalidated(
        &p,
        "T.cs",
        "public class T {\n    public int Compute(int x) { return x * 2; }\n}\n",
        "public class T {\n    public int Compute(int x) {\n        // double it\n        return x * 2;\n    }\n}\n",
        "Compute",
    );
}

// ── Go ─────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_go() {
    use ecp_analyzer::go::parser::GoProvider;
    let p = GoProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.go",
        "package main\n\nfunc compute(x int) int { return x * 2 }\n",
        "package main\n\nfunc compute(x int) int {\n\t// double it\n\treturn x * 2\n}\n",
        "compute",
    );
}

// ── Rust ───────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_rust() {
    use ecp_analyzer::rust::parser::RustProvider;
    let p = RustProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.rs",
        "pub fn compute(x: i32) -> i32 { x * 2 }\n",
        "pub fn compute(x: i32) -> i32 {\n    // double it\n    x * 2\n}\n",
        "compute",
    );
}

// ── PHP ────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_php() {
    use ecp_analyzer::php::parser::PhpProvider;
    let p = PhpProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.php",
        "<?php\nfunction compute($x) { return $x * 2; }\n",
        "<?php\nfunction compute($x) {\n    // double it\n    return $x * 2;\n}\n",
        "compute",
    );
}

// ── Ruby ───────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_ruby() {
    use ecp_analyzer::ruby::parser::RubyProvider;
    let p = RubyProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.rb",
        "def compute(x)\n  x * 2\nend\n",
        "def compute(x)\n  # double it\n  x * 2\nend\n",
        "compute",
    );
}

// ── Swift ──────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_swift() {
    use ecp_analyzer::swift::parser::SwiftProvider;
    let p = SwiftProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.swift",
        "func compute(_ x: Int) -> Int { return x * 2 }\n",
        "func compute(_ x: Int) -> Int {\n    // double it\n    return x * 2\n}\n",
        "compute",
    );
}

// ── C ──────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_c() {
    use ecp_analyzer::c::parser::CProvider;
    let p = CProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.c",
        "int compute(int x) { return x * 2; }\n",
        "int compute(int x) {\n    /* double it */\n    return x * 2;\n}\n",
        "compute",
    );
}

// ── C++ ────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_cpp() {
    use ecp_analyzer::cpp::parser::CppProvider;
    let p = CppProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.cpp",
        "int compute(int x) { return x * 2; }\n",
        "int compute(int x) {\n    // double it\n    return x * 2;\n}\n",
        "compute",
    );
}

// ── Dart ───────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_invalidation_dart() {
    use ecp_analyzer::dart::parser::DartProvider;
    let p = DartProvider::new().unwrap();
    assert_invalidated(
        &p,
        "t.dart",
        "int compute(int x) => x * 2;\n",
        "int compute(int x) {\n  // double it\n  return x * 2;\n}\n",
        "compute",
    );
}
