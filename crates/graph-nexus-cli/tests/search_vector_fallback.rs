//! Regression: vector + hybrid modes must degrade to BM25 (not crash)
//! when `graph.embeddings == None`. Mirrors the hook contract that
//! search never errors out — the cost of an unindexed graph is a
//! stderr warning, not a missing tool result.

use graph_nexus_cli::commands::search::{compute_hits, SearchArgs, SearchMode};
use graph_nexus_cli::engine::Engine;
use graph_nexus_core::graph::{
    File, FileCategory, Node, NodeKind, ZeroCopyGraph, GRAPH_FORMAT_VERSION, GRAPH_MAGIC,
};
use graph_nexus_core::pool::StringPool;
use rkyv::rancor::Error;
use std::fs;
use tempfile::tempdir;

fn make_minimal_graph_no_embeddings() -> ZeroCopyGraph {
    let mut pool = StringPool::new();
    let file_ref = pool.add("src/lib.rs");
    let name_ref = pool.add("validateUser");
    let uid_ref = pool.add("Function:src/lib.rs:validateUser");
    let nodes = vec![Node {
        uid: uid_ref,
        name: name_ref,
        file_idx: 0,
        kind: NodeKind::Function,
        span: (0, 0, 1, 0),
        community_id: 0,
    }];
    ZeroCopyGraph {
        magic: GRAPH_MAGIC,
        version: GRAPH_FORMAT_VERSION,
        fingerprint: [0; 32],
        string_pool: pool.bytes,
        files: vec![File {
            path: file_ref,
            mtime: 0,
            content_hash: [0; 32],
            category: FileCategory::Source,
        }],
        nodes,
        edges: vec![],
        out_offsets: vec![0, 0],
        in_offsets: vec![0, 0],
        in_edge_idx: vec![],
        name_index: vec![],
        embeddings: None,
        process_start: 1,
        traces_offsets: vec![],
        traces_data: vec![],
        blind_spots: vec![],
        route_shapes: vec![],
    }
}

fn write_graph_and_load(graph: ZeroCopyGraph) -> (tempfile::TempDir, Engine) {
    let dir = tempdir().unwrap();
    let graph_path = dir.path().join("graph.bin");
    let bytes = rkyv::to_bytes::<Error>(&graph).unwrap();
    fs::write(&graph_path, bytes).unwrap();
    let engine = Engine::load(graph_path).unwrap();
    (dir, engine)
}

#[test]
fn vector_falls_back_to_bm25_when_embeddings_missing() {
    let (_dir, engine) = write_graph_and_load(make_minimal_graph_no_embeddings());
    let args = SearchArgs {
        pattern: Some("validateUser".into()),
        mode: SearchMode::Vector,
        kind: None,
        repo: None,
        format: None,
        batch: false,
    };
    let hits = compute_hits(args, &engine).expect("compute_hits Err");
    assert!(
        hits.iter().any(|h| h.name == "validateUser"),
        "expected BM25 fallback to surface validateUser, got {:?}",
        hits.iter().map(|h| &h.name).collect::<Vec<_>>()
    );
}

#[test]
fn hybrid_falls_back_to_bm25_when_embeddings_missing() {
    let (_dir, engine) = write_graph_and_load(make_minimal_graph_no_embeddings());
    let args = SearchArgs {
        pattern: Some("validateUser".into()),
        mode: SearchMode::Hybrid,
        kind: None,
        repo: None,
        format: None,
        batch: false,
    };
    let hits = compute_hits(args, &engine).expect("compute_hits Err");
    assert!(
        hits.iter().any(|h| h.name == "validateUser"),
        "expected hybrid → BM25 fallback to surface validateUser"
    );
}
