//! T7-2: Per-symbol content_hash stability.
//!
//! Parses the same source twice for each of the 14 mainstream languages and
//! asserts that every top-level symbol's `content_hash` is identical across
//! runs. A non-zero hash is also asserted so that a default-zeroed field is
//! caught immediately.

use std::path::Path;

use ecp_core::analyzer::provider::LanguageProvider;

// ── helpers ────────────────────────────────────────────────────────────────

fn hashes_for(provider: &dyn LanguageProvider, path: &str, src: &str) -> Vec<(String, u64)> {
    let lg = provider
        .parse_file(Path::new(path), src.as_bytes())
        .unwrap_or_else(|e| panic!("parse_file({path}) failed: {e}"));
    lg.nodes
        .iter()
        .map(|n| (n.name.clone(), n.content_hash))
        .collect()
}

fn assert_stable(provider: &dyn LanguageProvider, path: &str, src: &str) {
    let run1 = hashes_for(provider, path, src);
    let run2 = hashes_for(provider, path, src);
    assert_eq!(
        run1, run2,
        "{path}: content_hash must be identical across two parses of identical source"
    );
    for (name, hash) in &run1 {
        assert_ne!(
            *hash, 0,
            "{path}: symbol `{name}` has content_hash=0 (placeholder not replaced)"
        );
    }
}

// ── TypeScript ──────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_typescript() {
    use ecp_analyzer::typescript::parser::TypeScriptProvider;
    let p = TypeScriptProvider::new().unwrap();
    assert_stable(
        &p,
        "t.ts",
        "export function greet(name: string): string {\n  return `Hello, ${name}`;\n}\n",
    );
}

// ── JavaScript ─────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_javascript() {
    use ecp_analyzer::javascript::parser::JavaScriptProvider;
    let p = JavaScriptProvider::new().unwrap();
    assert_stable(&p, "t.js", "function add(a, b) { return a + b; }\n");
}

// ── Python ─────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_python() {
    use ecp_analyzer::python::parser::PythonProvider;
    let p = PythonProvider::new().unwrap();
    assert_stable(&p, "t.py", "def compute(x, y):\n    return x + y\n");
}

// ── Java ───────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_java() {
    use ecp_analyzer::java::parser::JavaProvider;
    let p = JavaProvider::new().unwrap();
    assert_stable(
        &p,
        "T.java",
        "public class T {\n    public int add(int a, int b) { return a + b; }\n}\n",
    );
}

// ── Kotlin ─────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_kotlin() {
    use ecp_analyzer::kotlin::parser::KotlinProvider;
    let p = KotlinProvider::new().unwrap();
    assert_stable(
        &p,
        "t.kt",
        "fun greet(name: String): String = \"Hello, $name\"\n",
    );
}

// ── C# ─────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_csharp() {
    use ecp_analyzer::c_sharp::parser::CSharpProvider;
    let p = CSharpProvider::new().unwrap();
    assert_stable(
        &p,
        "T.cs",
        "public class T {\n    public int Add(int a, int b) => a + b;\n}\n",
    );
}

// ── Go ─────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_go() {
    use ecp_analyzer::go::parser::GoProvider;
    let p = GoProvider::new().unwrap();
    assert_stable(
        &p,
        "t.go",
        "package main\n\nfunc add(a, b int) int { return a + b }\n",
    );
}

// ── Rust ───────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_rust() {
    use ecp_analyzer::rust::parser::RustProvider;
    let p = RustProvider::new().unwrap();
    assert_stable(&p, "t.rs", "pub fn add(a: i32, b: i32) -> i32 { a + b }\n");
}

// ── PHP ────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_php() {
    use ecp_analyzer::php::parser::PhpProvider;
    let p = PhpProvider::new().unwrap();
    assert_stable(
        &p,
        "t.php",
        "<?php\nfunction add($a, $b) { return $a + $b; }\n",
    );
}

// ── Ruby ───────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_ruby() {
    use ecp_analyzer::ruby::parser::RubyProvider;
    let p = RubyProvider::new().unwrap();
    assert_stable(&p, "t.rb", "def add(a, b)\n  a + b\nend\n");
}

// ── Swift ──────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_swift() {
    use ecp_analyzer::swift::parser::SwiftProvider;
    let p = SwiftProvider::new().unwrap();
    assert_stable(
        &p,
        "t.swift",
        "func add(_ a: Int, _ b: Int) -> Int { return a + b }\n",
    );
}

// ── C ──────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_c() {
    use ecp_analyzer::c::parser::CProvider;
    let p = CProvider::new().unwrap();
    assert_stable(&p, "t.c", "int add(int a, int b) { return a + b; }\n");
}

// ── C++ ────────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_cpp() {
    use ecp_analyzer::cpp::parser::CppProvider;
    let p = CppProvider::new().unwrap();
    assert_stable(&p, "t.cpp", "int add(int a, int b) { return a + b; }\n");
}

// ── Dart ───────────────────────────────────────────────────────────────────

#[test]
fn test_content_hash_stability_dart() {
    use ecp_analyzer::dart::parser::DartProvider;
    let p = DartProvider::new().unwrap();
    assert_stable(&p, "t.dart", "int add(int a, int b) => a + b;\n");
}
