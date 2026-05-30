//! End-to-end: a .sql migration defines a table (Class node); Python code
//! queries it via a raw SQL string; the built graph must contain a
//! QueriesTable edge from the function to the table, with read/write reason.
//!
//! Exercises the full pipeline: SQL DDL parser emits Class node → Python
//! parser emits RawSqlRef → GraphBuilder::build() → sql_table_edges::emit_edges
//! → ZeroCopyGraph with QueriesTable edges.
//!
//! Dynamic/interpolated table names (f-strings with {var}) are skipped by
//! the Python extractor — no RawSqlRef emitted — so no fabricated edge appears.

use ecp_analyzer::python::parser::PythonProvider;
use ecp_analyzer::resolution::builder::GraphBuilder;
use ecp_analyzer::sql::parser::SqlProvider;
use ecp_core::analyzer::provider::LanguageProvider;
use ecp_core::analyzer::types::LocalGraph;
use ecp_core::graph::{RelType, ZeroCopyGraph};

fn parse_py(path: &str, src: &str) -> LocalGraph {
    PythonProvider::new()
        .expect("python provider")
        .parse_file(path.as_ref(), src.as_bytes())
        .expect("parse_file")
}

fn parse_sql(path: &str, src: &str) -> LocalGraph {
    SqlProvider::new()
        .expect("sql provider")
        .parse_file(path.as_ref(), src.as_bytes())
        .expect("parse_file")
}

fn build(lgs: Vec<LocalGraph>) -> ZeroCopyGraph {
    let mut b = GraphBuilder::new();
    for lg in lgs {
        b.add_graph(lg);
    }
    b.build()
}

/// Returns true if a QueriesTable edge exists src_name -> tgt_name with the
/// given reason string. Walks graph.edges, filters by QueriesTable rel_type,
/// resolves source/target node names and edge reason from the string pool.
fn has_queries_table_edge(g: &ZeroCopyGraph, src_name: &str, tgt_name: &str, reason: &str) -> bool {
    let pool = g.string_pool.as_slice();
    for e in &g.edges {
        if e.rel_type != RelType::QueriesTable {
            continue;
        }
        let src_node_name = g.nodes[e.source as usize].name.resolve(pool);
        let tgt_node_name = g.nodes[e.target as usize].name.resolve(pool);
        let edge_reason = e.reason.resolve(pool);
        if src_node_name == src_name && tgt_node_name == tgt_name && edge_reason == reason {
            return true;
        }
    }
    false
}

/// Count QueriesTable edges whose source node name equals `src_name`.
fn count_queries_table_edges_from(g: &ZeroCopyGraph, src_name: &str) -> usize {
    let pool = g.string_pool.as_slice();
    g.edges
        .iter()
        .filter(|e| {
            e.rel_type == RelType::QueriesTable
                && g.nodes[e.source as usize].name.resolve(pool) == src_name
        })
        .count()
}

/// Debug helper: print all QueriesTable edges found (for --nocapture diagnostics).
fn print_queries_table_edges(g: &ZeroCopyGraph) {
    let pool = g.string_pool.as_slice();
    let found: Vec<_> = g
        .edges
        .iter()
        .filter(|e| e.rel_type == RelType::QueriesTable)
        .map(|e| {
            (
                g.nodes[e.source as usize].name.resolve(pool).to_owned(),
                g.nodes[e.target as usize].name.resolve(pool).to_owned(),
                e.reason.resolve(pool).to_owned(),
            )
        })
        .collect();
    eprintln!("QueriesTable edges ({} total):", found.len());
    for (src, tgt, reason) in &found {
        eprintln!("  {} -> {} (reason={})", src, tgt, reason);
    }
}

#[test]
fn select_emits_queries_table_read_edge_end_to_end() {
    let sql = parse_sql(
        "schema.sql",
        "CREATE TABLE channels (id BIGINT PRIMARY KEY, slug TEXT);",
    );
    let py = parse_py(
        "api.py",
        "def list_channels(pool):\n    return pool.fetch(\"SELECT id, slug FROM channels WHERE org_id = $1\")\n",
    );
    let g = build(vec![sql, py]);
    print_queries_table_edges(&g);
    assert!(
        has_queries_table_edge(&g, "list_channels", "channels", "read"),
        "expected QueriesTable(read) list_channels -> channels"
    );
}

#[test]
fn update_emits_queries_table_write_edge_end_to_end() {
    let sql = parse_sql(
        "schema.sql",
        "CREATE TABLE channels (id BIGINT PRIMARY KEY, slug TEXT);",
    );
    let py = parse_py(
        "svc.py",
        "def rename(pool):\n    pool.execute(\"UPDATE channels SET slug = $1 WHERE id = $2\")\n",
    );
    let g = build(vec![sql, py]);
    print_queries_table_edges(&g);
    assert!(
        has_queries_table_edge(&g, "rename", "channels", "write"),
        "expected QueriesTable(write) rename -> channels"
    );
}

#[test]
fn dynamic_table_name_produces_no_fabricated_edge() {
    let sql = parse_sql(
        "schema.sql",
        "CREATE TABLE channels (id BIGINT PRIMARY KEY);",
    );
    // f-string with {tbl} interpolation: the Python extractor skips f-strings
    // that have `interpolation` children, so no RawSqlRef is emitted at all.
    // No RawSqlRef → no QueriesTable edge → no fabrication.
    let py = parse_py(
        "svc.py",
        "def run(pool, tbl):\n    pool.execute(f\"SELECT * FROM {tbl} WHERE id = $1\")\n",
    );
    let g = build(vec![sql, py]);
    print_queries_table_edges(&g);
    let count = count_queries_table_edges_from(&g, "run");
    assert_eq!(
        count, 0,
        "dynamic table name must not fabricate a QueriesTable edge"
    );
}
