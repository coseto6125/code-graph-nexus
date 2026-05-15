use crate::cypher::ast::Query;
use crate::cypher::error::CypherError;
use crate::cypher::value::{QueryResult, Value};
use crate::graph::ArchivedZeroCopyGraph;
use std::collections::HashMap;
use std::path::{Path, PathBuf};

/// One row of intermediate bindings during pattern matching.
#[derive(Debug, Clone, Default)]
struct Binding {
    /// var_name -> node index into `graph.nodes`
    node_vars: HashMap<String, u32>,
    /// var_name -> edge index into `graph.edges`
    edge_vars: HashMap<String, u32>,
}

/// Reading file content for `.content` projection (used by C12+).
#[allow(dead_code)]
struct ContentCache {
    repo_root: PathBuf,
    files: HashMap<u32, Option<String>>,
}

impl ContentCache {
    fn new(repo_root: PathBuf) -> Self {
        Self {
            repo_root,
            files: HashMap::new(),
        }
    }

    #[allow(dead_code)]
    fn body_for_file(&mut self, graph: &ArchivedZeroCopyGraph, file_idx: u32) -> Option<&str> {
        if !self.files.contains_key(&file_idx) {
            let body = if (file_idx as usize) < graph.files.len() {
                let rel = graph.files[file_idx as usize]
                    .path
                    .resolve(&graph.string_pool);
                std::fs::read_to_string(self.repo_root.join(rel)).ok()
            } else {
                None
            };
            self.files.insert(file_idx, body);
        }
        self.files.get(&file_idx).and_then(|o| o.as_deref())
    }
}

pub fn execute(
    query: &Query,
    graph: &ArchivedZeroCopyGraph,
    repo_root: &Path,
) -> Result<QueryResult, CypherError> {
    let mut cache = ContentCache::new(repo_root.to_path_buf());
    execute_inner(query, graph, &mut cache)
}

fn execute_inner(
    _query: &Query,
    _graph: &ArchivedZeroCopyGraph,
    _cache: &mut ContentCache,
) -> Result<QueryResult, CypherError> {
    Err(CypherError::Exec {
        msg: "executor not yet wired".into(),
    })
}

// ---------------------------------------------------------------------------
// C1 scaffold test
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn scaffolding_compiles() {
        let _c = ContentCache::new(PathBuf::from("."));
        let _b = Binding::default();
        assert!((_b.node_vars).is_empty());
    }
}
