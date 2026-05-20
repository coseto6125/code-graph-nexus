//! Snapshot regression tests for the three SFC parser fixtures.
//!
//! Fixtures live in `scripts/parity/sfc_fixtures/`. Node counts are pinned
//! against `scripts/parity/sfc_snapshot_baseline.txt`.
//!
//! Oracle asymmetry: ref-gitnexus supports Vue but NOT Svelte or Astro.
//! Vue counts can be cross-checked against ref-gitnexus; Svelte and Astro
//! are snapshot-only. Section/Folder/File/Document nodes are ecp-only
//! (excluded from ref-gitnexus A/B delta per the inclusive-emission policy).

use ecp_analyzer::{
    astro::parser::AstroProvider, svelte::parser::SvelteProvider, vue::parser::VueProvider,
};
use ecp_core::{analyzer::provider::LanguageProvider, graph::NodeKind};
use std::path::Path;

fn ws_root() -> std::path::PathBuf {
    std::path::PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("workspace root")
        .to_path_buf()
}

fn fixture(name: &str) -> Vec<u8> {
    let path = ws_root().join("scripts/parity/sfc_fixtures").join(name);
    std::fs::read(&path).unwrap_or_else(|_| panic!("fixture missing: {path:?}"))
}

fn count_kind(graph: &ecp_core::analyzer::types::LocalGraph, kind: NodeKind) -> usize {
    graph.nodes.iter().filter(|n| n.kind == kind).count()
}

// ── Vue: Component.vue ───────────────────────────────────────────────────────
//
// Baseline (scripts/parity/sfc_snapshot_baseline.txt):
//   total_nodes: 6  imports: 2
//   Section: 3, Const: 1, Function: 2
//
// ref-gitnexus delta (Section excluded): Const:1 + Function:2 = 3 comparable nodes.
// Acceptable gap vs oracle: ≤15% — 3 nodes is at the low end; delta must be 0
// here because this is a tiny controlled fixture with no ambiguous constructs.

#[test]
fn vue_component_snapshot() {
    let src = fixture("Component.vue");
    let graph = VueProvider::new()
        .expect("VueProvider::new")
        .parse_file(Path::new("Component.vue"), &src)
        .expect("parse_file");

    assert_eq!(
        graph.nodes.len(),
        6,
        "Vue Component.vue: total node count changed"
    );
    assert_eq!(
        graph.imports.len(),
        2,
        "Vue Component.vue: import count changed"
    );

    assert_eq!(
        count_kind(&graph, NodeKind::Section),
        3,
        "Vue: Section count"
    );
    assert_eq!(count_kind(&graph, NodeKind::Const), 1, "Vue: Const count");
    assert_eq!(
        count_kind(&graph, NodeKind::Function),
        2,
        "Vue: Function count"
    );

    let fns: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Function)
        .map(|n| n.name.as_str())
        .collect();
    assert!(
        fns.contains(&"increment"),
        "Vue: increment function missing"
    );
    assert!(
        fns.contains(&"decrement"),
        "Vue: decrement function missing"
    );

    let sections: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Section)
        .map(|n| n.name.as_str())
        .collect();
    assert!(
        sections.contains(&"template"),
        "Vue: template section missing"
    );
    assert!(
        sections.contains(&"script setup"),
        "Vue: script setup section missing"
    );
    assert!(sections.contains(&"style"), "Vue: style section missing");

    let import_srcs: Vec<&str> = graph.imports.iter().map(|i| i.source.as_str()).collect();
    assert!(
        import_srcs.contains(&"vue"),
        "Vue: import from 'vue' missing"
    );
    assert!(
        import_srcs.contains(&"../composables/useStore"),
        "Vue: useStore import missing"
    );
}

// ── Svelte: Counter.svelte (snapshot-only) ───────────────────────────────────
//
// Baseline (scripts/parity/sfc_snapshot_baseline.txt):
//   total_nodes: 4  imports: 1
//   Section: 2, Const: 1, Function: 1

#[test]
fn svelte_counter_snapshot() {
    let src = fixture("Counter.svelte");
    let graph = SvelteProvider::new()
        .expect("SvelteProvider::new")
        .parse_file(Path::new("Counter.svelte"), &src)
        .expect("parse_file");

    assert_eq!(
        graph.nodes.len(),
        4,
        "Svelte Counter.svelte: total node count changed"
    );
    assert_eq!(
        graph.imports.len(),
        1,
        "Svelte Counter.svelte: import count changed"
    );

    assert_eq!(
        count_kind(&graph, NodeKind::Section),
        2,
        "Svelte: Section count"
    );
    assert_eq!(
        count_kind(&graph, NodeKind::Const),
        1,
        "Svelte: Const count"
    );
    assert_eq!(
        count_kind(&graph, NodeKind::Function),
        1,
        "Svelte: Function count"
    );

    let fns: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Function)
        .map(|n| n.name.as_str())
        .collect();
    assert!(
        fns.contains(&"increment"),
        "Svelte: increment function missing"
    );

    let import_srcs: Vec<&str> = graph.imports.iter().map(|i| i.source.as_str()).collect();
    assert!(
        import_srcs.contains(&"svelte"),
        "Svelte: import from 'svelte' missing"
    );
}

// ── Astro: Page.astro (snapshot-only) ────────────────────────────────────────
//
// Baseline (scripts/parity/sfc_snapshot_baseline.txt):
//   total_nodes: 4  imports: 2
//   Section: 2, Interface: 1, Const: 1

#[test]
fn astro_page_snapshot() {
    let src = fixture("Page.astro");
    let graph = AstroProvider::new()
        .expect("AstroProvider::new")
        .parse_file(Path::new("Page.astro"), &src)
        .expect("parse_file");

    assert_eq!(
        graph.nodes.len(),
        4,
        "Astro Page.astro: total node count changed"
    );
    assert_eq!(
        graph.imports.len(),
        2,
        "Astro Page.astro: import count changed"
    );

    assert_eq!(
        count_kind(&graph, NodeKind::Section),
        2,
        "Astro: Section count"
    );
    assert_eq!(
        count_kind(&graph, NodeKind::Interface),
        1,
        "Astro: Interface count"
    );
    assert_eq!(count_kind(&graph, NodeKind::Const), 1, "Astro: Const count");

    let sections: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Section)
        .map(|n| n.name.as_str())
        .collect();
    assert!(
        sections.contains(&"frontmatter"),
        "Astro: frontmatter section missing"
    );
    assert!(
        sections.contains(&"template"),
        "Astro: template section missing"
    );

    let interfaces: Vec<&str> = graph
        .nodes
        .iter()
        .filter(|n| n.kind == NodeKind::Interface)
        .map(|n| n.name.as_str())
        .collect();
    assert!(
        interfaces.contains(&"User"),
        "Astro: User interface missing"
    );

    let import_srcs: Vec<&str> = graph.imports.iter().map(|i| i.source.as_str()).collect();
    assert!(
        import_srcs.contains(&"../layouts/Base.astro"),
        "Astro: Layout import missing"
    );
    assert!(
        import_srcs.contains(&"../api/users"),
        "Astro: fetchUsers import missing"
    );
}
