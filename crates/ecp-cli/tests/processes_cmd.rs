//! Integration tests for `ecp processes` (FU-013).
//!
//! Mirrors the synthetic-graph pattern from `find_cmd.rs`: build a tiny
//! `ZeroCopyGraph` containing one Process node + 3 member Functions,
//! serialise to `graph.bin`, then spawn `ecp processes ...` against it
//! via `--graph <path>`.
//!
//! The CLI surface is one top-level command + one subcommand (`trace`).
//! 14-language coverage doesn't apply — Process emission is post-process,
//! parser-agnostic (driven by Calls edges).

use ecp_core::graph::{
    Edge, File, FileCategory, Node, NodeKind, RelType, ZeroCopyGraph, GRAPH_FORMAT_VERSION,
    GRAPH_MAGIC,
};
use ecp_core::pool::{StrRef, StringPool};
use rkyv::rancor::Error;
use serde_json::Value;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::TempDir;

fn ecp_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ecp")
}

/// Build a graph with `member_count` Function members + 1 Process node
/// whose trace covers all members. `process_label` is the Process name
/// (`"Entry → Terminal"` shape). `communities` aligns 1-1 with members;
/// pass distinct values to exercise the cross-vs-intra classifier.
fn build_process_graph(
    process_label: &str,
    member_names: &[&str],
    communities: &[u16],
) -> (TempDir, PathBuf) {
    assert_eq!(
        member_names.len(),
        communities.len(),
        "fixture length mismatch"
    );
    let dir = TempDir::new().unwrap();
    let mut pool = StringPool::new();

    let file_path = pool.add("src/lib.rs");
    let files = vec![File {
        path: file_path,
        mtime: 0,
        content_hash: [0; 8],
        category: FileCategory::Source,
    }];

    let mut nodes: Vec<Node> = member_names
        .iter()
        .zip(communities.iter())
        .enumerate()
        .map(|(i, (name, comm))| Node {
            uid: ecp_core::uid::compute(NodeKind::Function, "src/lib.rs", None, name),
            name: pool.add(name),
            file_idx: 0,
            kind: NodeKind::Function,
            span: ((i * 10) as u32 + 1, 0, (i * 10) as u32 + 5, 0),
            community_id: *comm,
            owner_class: StrRef::default(),
            content_hash: 0,
        })
        .collect();

    let process_start = nodes.len() as u32;
    let process_community = communities[0];
    nodes.push(Node {
        uid: ecp_core::uid::compute(NodeKind::Process, "src/lib.rs", None, process_label),
        name: pool.add(process_label),
        file_idx: 0,
        kind: NodeKind::Process,
        span: (1, 0, 5, 0),
        community_id: process_community,
        owner_class: StrRef::default(),
        content_hash: 0,
    });

    let n = nodes.len();
    let process_idx = process_start;
    let reason = pool.add("step:test");
    let edges: Vec<Edge> = (0..member_names.len() as u32)
        .map(|i| Edge {
            source: i,
            target: process_idx,
            rel_type: RelType::StepInProcess,
            confidence: 1.0,
            reason,
        })
        .collect();

    let out_offsets = vec![0u32; n + 1];
    let in_offsets = vec![0u32; n + 1];
    let in_edge_idx: Vec<u32> = Vec::new();

    let traces_data: Vec<u32> = (0..member_names.len() as u32).collect();
    let traces_offsets = vec![0u32, traces_data.len() as u32];

    let graph = ZeroCopyGraph {
        magic: GRAPH_MAGIC,
        version: GRAPH_FORMAT_VERSION,
        fingerprint: [0; 32],
        string_pool: pool.bytes,
        files,
        nodes,
        edges,
        out_offsets,
        in_offsets,
        in_edge_idx,
        name_index: Vec::new(),
        process_start,
        traces_offsets,
        traces_data,
        blind_spots: vec![],
        route_shapes: vec![],
        call_metas: vec![],
        function_metas: vec![],
        kind_offsets: vec![],
        kind_node_idx: vec![],
        node_flags: vec![],
    };

    let bytes = rkyv::to_bytes::<Error>(&graph).unwrap();
    let graph_path = dir.path().join("graph.bin");
    std::fs::write(&graph_path, &bytes).unwrap();
    (dir, graph_path)
}

fn run_processes(graph: &Path, args: &[&str]) -> std::process::Output {
    Command::new(ecp_bin())
        .arg("processes")
        .args(args)
        .arg("--graph")
        .arg(graph)
        .arg("--format")
        .arg("json")
        .output()
        .expect("ecp processes spawn")
}

fn parse_json_stdout(out: &std::process::Output) -> Value {
    let stdout = String::from_utf8_lossy(&out.stdout);
    let start = stdout
        .find('{')
        .unwrap_or_else(|| panic!("no JSON in stdout: {stdout}"));
    serde_json::from_str(&stdout[start..])
        .unwrap_or_else(|e| panic!("JSON parse error: {e}\nstdout: {stdout}"))
}

#[test]
fn list_returns_process_label_and_step_count() {
    let (_dir, graph) = build_process_graph(
        "Authenticate → IssueToken",
        &["authenticate", "verify_password", "issue_token"],
        &[1, 1, 1],
    );

    let out = run_processes(&graph, &[]);
    assert!(
        out.status.success(),
        "ecp processes exited non-zero: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload = parse_json_stdout(&out);
    assert_eq!(payload["status"], "success");
    assert_eq!(payload["total"], 1);
    let results = payload["results"].as_array().expect("results array");
    assert_eq!(results.len(), 1);
    assert_eq!(results[0]["label"], "Authenticate → IssueToken");
    assert_eq!(results[0]["step_count"], 3);
    assert_eq!(results[0]["process_type"], "intra_community");
}

#[test]
fn list_cross_community_classification() {
    let (_dir, graph) = build_process_graph(
        "EntryFn → TerminalFn",
        &["entry_fn", "middle_fn", "terminal_fn"],
        &[1, 2, 1], // distinct communities → cross
    );
    let payload = parse_json_stdout(&run_processes(&graph, &[]));
    assert_eq!(payload["results"][0]["process_type"], "cross_community");
}

#[test]
fn trace_substring_match_emits_ordered_steps() {
    let (_dir, graph) = build_process_graph(
        "HandleRequest → WriteResponse",
        &["handle_request", "parse_body", "write_response"],
        &[1, 1, 1],
    );

    // Match by substring of the label (case-insensitive).
    let out = run_processes(&graph, &["trace", "handlerequest"]);
    assert!(
        out.status.success(),
        "ecp processes trace exited non-zero: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload = parse_json_stdout(&out);
    assert_eq!(payload["status"], "success");
    assert_eq!(payload["matched"], 1);

    let steps = payload["results"][0]["steps"]
        .as_array()
        .expect("steps array");
    assert_eq!(steps.len(), 3);
    assert_eq!(steps[0]["name"], "handle_request");
    assert_eq!(steps[1]["name"], "parse_body");
    assert_eq!(steps[2]["name"], "write_response");
    assert_eq!(steps[0]["step"], 1);
    assert_eq!(steps[2]["step"], 3);
}

#[test]
fn trace_no_match_returns_not_found() {
    let (_dir, graph) = build_process_graph("Foo → Bar", &["foo", "mid", "bar"], &[1, 1, 1]);
    let out = run_processes(&graph, &["trace", "nonexistent-process-pattern"]);
    assert!(out.status.success());
    let payload = parse_json_stdout(&out);
    assert_eq!(payload["status"], "not_found");
}

/// Build a graph where non-Process nodes (mimicking the PathLiteral / File
/// nodes that later builder passes append) follow the single Process node.
/// This breaks the "everything after process_start is a Process" assumption,
/// which used to make `processes` index `traces_offsets[k+1]` out of bounds
/// once the limit reached past the real process count (a small real repo:
/// `total` was over-counted as `nodes.len() - process_start`, and listing
/// with `--limit` ≥ that miscount panicked at `traces_offsets[k+1]`).
fn build_graph_with_trailing_non_process_nodes() -> (TempDir, PathBuf) {
    let dir = TempDir::new().unwrap();
    let mut pool = StringPool::new();
    let file_path = pool.add("src/lib.rs");
    let files = vec![File {
        path: file_path,
        mtime: 0,
        content_hash: [0; 8],
        category: FileCategory::Source,
    }];

    let member_names = ["entry", "middle", "terminal"];
    let mut nodes: Vec<Node> = member_names
        .iter()
        .enumerate()
        .map(|(i, name)| Node {
            uid: ecp_core::uid::compute(NodeKind::Function, "src/lib.rs", None, name),
            name: pool.add(name),
            file_idx: 0,
            kind: NodeKind::Function,
            span: ((i * 10) as u32 + 1, 0, (i * 10) as u32 + 5, 0),
            community_id: 1,
            owner_class: StrRef::default(),
            content_hash: 0,
        })
        .collect();

    let process_start = nodes.len() as u32;
    nodes.push(Node {
        uid: ecp_core::uid::compute(NodeKind::Process, "src/lib.rs", None, "Entry → Terminal"),
        name: pool.add("Entry → Terminal"),
        file_idx: 0,
        kind: NodeKind::Process,
        span: (1, 0, 5, 0),
        community_id: 1,
        owner_class: StrRef::default(),
        content_hash: 0,
    });

    // Trailing non-Process nodes after the single Process — the regression: a
    // naive `nodes.len() - process_start` would count these as processes.
    for i in 0..20 {
        nodes.push(Node {
            uid: ecp_core::uid::compute(
                NodeKind::PathLiteral,
                "src/lib.rs",
                None,
                &format!("p{i}"),
            ),
            name: pool.add(&format!("path/{i}")),
            file_idx: 0,
            kind: NodeKind::PathLiteral,
            span: (1, 0, 1, 0),
            community_id: 0,
            owner_class: StrRef::default(),
            content_hash: 0,
        });
    }

    let n = nodes.len();
    let traces_data: Vec<u32> = (0..member_names.len() as u32).collect();
    let traces_offsets = vec![0u32, traces_data.len() as u32];

    let graph = ZeroCopyGraph {
        magic: GRAPH_MAGIC,
        version: GRAPH_FORMAT_VERSION,
        fingerprint: [0; 32],
        string_pool: pool.bytes,
        files,
        nodes,
        edges: Vec::new(),
        out_offsets: vec![0u32; n + 1],
        in_offsets: vec![0u32; n + 1],
        in_edge_idx: Vec::new(),
        name_index: Vec::new(),
        process_start,
        traces_offsets,
        traces_data,
        blind_spots: vec![],
        route_shapes: vec![],
        call_metas: vec![],
        function_metas: vec![],
        kind_offsets: vec![],
        kind_node_idx: vec![],
        node_flags: vec![],
    };

    let bytes = rkyv::to_bytes::<Error>(&graph).unwrap();
    let graph_path = dir.path().join("graph.bin");
    std::fs::write(&graph_path, &bytes).unwrap();
    (dir, graph_path)
}

#[test]
fn list_does_not_overcount_or_panic_with_trailing_non_process_nodes() {
    let (_dir, graph) = build_graph_with_trailing_non_process_nodes();
    // A limit far past the real process count (1) used to walk into the
    // trailing PathLiteral nodes and panic at `traces_offsets[k+1]`.
    let out = run_processes(&graph, &["--limit", "100"]);
    assert!(
        out.status.success(),
        "ecp processes panicked on trailing non-Process nodes: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload = parse_json_stdout(&out);
    assert_eq!(payload["status"], "success");
    // total is the true process count (1), not nodes.len() - process_start (21).
    assert_eq!(payload["total"], 1);
    assert_eq!(payload["shown"], 1);
}

#[test]
fn trace_does_not_panic_with_trailing_non_process_nodes() {
    let (_dir, graph) = build_graph_with_trailing_non_process_nodes();
    let out = run_processes(&graph, &["trace", "entry", "--limit", "100"]);
    assert!(
        out.status.success(),
        "ecp processes trace panicked: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let payload = parse_json_stdout(&out);
    assert_eq!(payload["status"], "success");
    assert_eq!(payload["matched"], 1);
}
