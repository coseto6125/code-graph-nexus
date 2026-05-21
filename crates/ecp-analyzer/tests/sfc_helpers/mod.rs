//! Shared assertion helpers for `tests/{vue,astro,svelte}_sfc.rs`.
//!
//! Each `<lang>_sfc.rs` is compiled as its own test binary; this module
//! is declared via `mod sfc_helpers;` at the top of each so the same
//! helpers do not have to be triple-pasted.

use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::{LocalGraph, RawNode};
use ecp_core::graph::NodeKind;
use std::path::Path;

/// Run `provider.parse_file` against `src` with `path` as the virtual
/// filename. Panics on grammar load or parse failure — these are
/// assertion helpers, not production code.
pub fn parse_with<P: LanguageProvider>(provider: P, path: &str, src: &str) -> LocalGraph {
    provider
        .parse_file(Path::new(path), src.as_bytes())
        .expect("parse_file")
}

/// Collect the names of every node with `kind` in `graph`, preserving
/// graph order.
pub fn node_names_by_kind(graph: &LocalGraph, kind: NodeKind) -> Vec<&str> {
    graph
        .nodes
        .iter()
        .filter(|n| n.kind == kind)
        .map(|n| n.name.as_str())
        .collect()
}

/// Locate a node by exact name; on miss, panic with the list of every
/// node name in the graph (so the failure message names what was
/// actually produced).
pub fn find_node<'a>(graph: &'a LocalGraph, name: &str) -> &'a RawNode {
    graph
        .nodes
        .iter()
        .find(|n| n.name == name)
        .unwrap_or_else(|| {
            let names: Vec<_> = graph.nodes.iter().map(|n| &n.name).collect();
            panic!("node `{name}` not found; graph contains: {names:#?}")
        })
}

/// Collect every import's source string in graph order.
pub fn import_sources(graph: &LocalGraph) -> Vec<&str> {
    graph.imports.iter().map(|i| i.source.as_str()).collect()
}
