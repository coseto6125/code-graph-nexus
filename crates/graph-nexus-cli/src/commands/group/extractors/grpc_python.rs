//! Python gRPC server-registration extractor: captures add_<Svc>Servicer_to_server calls.

use crate::commands::group::types::{
    ContractRole, ContractType, ExtractedContract, SymbolRef,
};
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

pub(super) const SERVICE_CONFIDENCE: f32 = 0.9;

/// Matches `<module>.add_<Svc>Servicer_to_server(impl, server)`.
/// The `attribute` node text is the full method name; we filter + strip in Rust.
const QUERY_SRC: &str = r#"
(call
  function: (attribute
    attribute: (identifier) @add_fn))
"#;

pub fn extract_grpc(file_path: &Path, source: &[u8]) -> Vec<ExtractedContract> {
    let mut parser = Parser::new();
    let lang: tree_sitter::Language = tree_sitter_python::LANGUAGE.into();
    if let Err(e) = parser.set_language(&lang) {
        tracing::warn!("group::extract_grpc (python): set_language failed: {e:?}");
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        tracing::warn!(
            "group::extract_grpc (python): parser.parse returned None for {}",
            file_path.display()
        );
        return Vec::new();
    };
    let query = match Query::new(&lang, QUERY_SRC) {
        Ok(q) => q,
        Err(e) => {
            tracing::warn!("group::extract_grpc (python): Query::new failed: {e:?}");
            return Vec::new();
        }
    };

    let fn_idx = match query.capture_index_for_name("add_fn") {
        Some(i) => i,
        None => return Vec::new(),
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source);
    let mut out: Vec<ExtractedContract> = Vec::new();

    while let Some(m) = matches.next() {
        let fn_name = capture_text(m, fn_idx, source);
        let Some(svc) = parse_add_fn(fn_name) else {
            continue;
        };
        out.push(ExtractedContract {
            contract_id: format!("grpc:{svc}:*"),
            contract_type: ContractType::Grpc,
            role: ContractRole::Provider,
            symbol_uid: format!("{}::add_{svc}Servicer_to_server", file_path.display()),
            symbol_ref: SymbolRef {
                file_path: file_path.display().to_string(),
                name: format!("add_{svc}Servicer_to_server"),
            },
            confidence: SERVICE_CONFIDENCE,
            service: None,
            meta: vec![("service".into(), svc.to_string())],
        });
    }
    out
}

/// Returns `Some("UserService")` for `"add_UserServiceServicer_to_server"`, `None` otherwise.
fn parse_add_fn(name: &str) -> Option<&str> {
    let after_add = name.strip_prefix("add_")?;
    let before_to = after_add.strip_suffix("_to_server")?;
    let svc = before_to.strip_suffix("Servicer")?;
    if svc.is_empty() {
        return None;
    }
    Some(svc)
}

fn capture_text<'a>(
    m: &tree_sitter::QueryMatch<'a, 'a>,
    idx: u32,
    source: &'a [u8],
) -> &'a str {
    for c in m.captures {
        if c.index == idx {
            return std::str::from_utf8(&source[c.node.byte_range()]).unwrap_or("");
        }
    }
    ""
}
