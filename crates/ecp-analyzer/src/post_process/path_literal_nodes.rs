//! `PathLiteral` Node emission + `UsesPathLiteral` edge from the enclosing
//! Function/Method captured by per-language `path_literals` extractors.
//!
//! Promotes each `LocalGraph.path_literals` entry to:
//!   - One `NodeKind::PathLiteral` Node with `name = literal value`,
//!     `span = literal span`, `owner_class = enclosing_owner.unwrap_or("")`.
//!   - One `UsesPathLiteral` Edge (enclosing function/method idx → PathLiteral
//!     idx) with `reason = pre-rendered sink_reason` from the extractor.
//!
//! Honest-no-data rule: when the enclosing symbol can't be resolved against
//! `SymbolTable` (module-top-level literals; partial-parse drops), the
//! PathLiteral Node is still emitted (so `MATCH (n:PathLiteral)` queries see
//! it) but the edge is skipped — no fabricated source.

use crate::resolution::index::SymbolTable;
use ecp_core::analyzer::types::LocalGraph;
use ecp_core::graph::{Edge, Node, NodeKind, RelType};
use ecp_core::pool::StringPool;
use ecp_core::uid;

/// Returns the number of `UsesPathLiteral` edges appended. PathLiteral node
/// count is derivable from `nodes.len()` delta and is logged separately by
/// the builder.
pub fn emit_edges(
    local_graphs: &[LocalGraph],
    symbol_table: &SymbolTable,
    string_pool: &mut StringPool,
    nodes: &mut Vec<Node>,
    edges: &mut Vec<Edge>,
) -> usize {
    let mut edge_count = 0usize;

    for (lg_idx, lg) in local_graphs.iter().enumerate() {
        let Some(ref literals) = lg.path_literals else {
            continue;
        };
        if literals.is_empty() {
            continue;
        }
        let path_str = lg.file_path.to_string_lossy().replace('\\', "/");
        let file_idx = lg_idx as u32;

        for raw in literals.iter() {
            let value = raw.value.as_str();
            let owner_name = raw.enclosing_owner.as_deref().unwrap_or("");

            // UID: include owner so two identical literals in different
            // methods of different impl blocks (same file) get distinct UIDs.
            let node_uid = uid::compute(
                NodeKind::PathLiteral,
                &path_str,
                if owner_name.is_empty() {
                    None
                } else {
                    Some(owner_name)
                },
                value,
            );
            let name_ref = string_pool.add(value);
            let owner_ref = string_pool.add(owner_name);

            let lit_idx = nodes.len() as u32;
            nodes.push(Node {
                uid: node_uid,
                name: name_ref,
                file_idx,
                kind: NodeKind::PathLiteral,
                span: raw.span,
                community_id: 0,
                owner_class: owner_ref,
                content_hash: 0,
            });

            let Some(enclosing_name) = raw.enclosing_symbol.as_deref() else {
                continue;
            };
            let Some(enclosing_idx) = symbol_table.lookup_in_file(&path_str, enclosing_name) else {
                continue;
            };

            let reason_ref = string_pool.add(&raw.sink_reason);
            edges.push(Edge {
                source: enclosing_idx,
                target: lit_idx,
                rel_type: RelType::UsesPathLiteral,
                confidence: 1.0,
                reason: reason_ref,
            });
            edge_count += 1;
        }
    }

    edge_count
}
