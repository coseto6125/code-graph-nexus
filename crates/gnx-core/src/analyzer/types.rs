use crate::graph::{NodeKind, RelType};
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct RawNode {
    pub name: String,
    pub kind: NodeKind,
    pub span: (u32, u32, u32, u32),
}

#[derive(Debug, Clone)]
pub struct RawImport {
    pub source: String,
    pub imported_name: String,
}

#[derive(Debug, Clone)]
pub struct LocalGraph {
    pub file_path: PathBuf,
    pub nodes: Vec<RawNode>,
    pub imports: Vec<RawImport>,
}
