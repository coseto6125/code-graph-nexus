//! Java HTTP route extractor: Spring MVC annotation patterns via tree-sitter.

use crate::commands::group::types::{
    ContractRole, ContractType, ExtractedContract, SymbolRef,
};
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

pub(super) const ROUTE_CONFIDENCE: f32 = 0.85;

/// Matches `@PostMapping("/path")` etc. inside a method_declaration.
/// `string_fragment` inside `string_literal` is already unquoted.
/// The method_declaration's name captures the handler symbol.
const QUERY_SRC: &str = r#"
(method_declaration
  (modifiers
    (annotation
      name: (identifier) @ann
      arguments: (annotation_argument_list
        (string_literal
          (string_fragment) @path))))
  name: (identifier) @handler)
"#;

pub fn extract_http(file_path: &Path, source: &[u8]) -> Vec<ExtractedContract> {
    let mut parser = Parser::new();
    let lang: tree_sitter::Language = tree_sitter_java::LANGUAGE.into();
    if let Err(e) = parser.set_language(&lang) {
        tracing::warn!("group::extract_http (java): set_language failed: {e:?}");
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        tracing::warn!(
            "group::extract_http (java): parser.parse returned None for {}",
            file_path.display()
        );
        return Vec::new();
    };
    let query = match Query::new(&lang, QUERY_SRC) {
        Ok(q) => q,
        Err(e) => {
            tracing::warn!("group::extract_http (java): Query::new failed: {e:?}");
            return Vec::new();
        }
    };

    let ann_idx = match query.capture_index_for_name("ann") {
        Some(i) => i,
        None => return Vec::new(),
    };
    let path_idx = match query.capture_index_for_name("path") {
        Some(i) => i,
        None => return Vec::new(),
    };
    let handler_idx = match query.capture_index_for_name("handler") {
        Some(i) => i,
        None => return Vec::new(),
    };

    let mut cursor = QueryCursor::new();
    let mut matches = cursor.matches(&query, tree.root_node(), source);
    let mut out: Vec<ExtractedContract> = Vec::new();

    while let Some(m) = matches.next() {
        let ann_name = capture_text(m, ann_idx, source);
        let Some(http_method) = http_method_from_annotation(ann_name) else {
            continue;
        };
        let route_path = capture_text(m, path_idx, source);
        let handler = capture_text(m, handler_idx, source);
        let id = format!("http:{http_method}:{route_path}");
        out.push(ExtractedContract {
            contract_id: id,
            contract_type: ContractType::Http,
            role: ContractRole::Provider,
            symbol_uid: format!("{}::{handler}", file_path.display()),
            symbol_ref: SymbolRef {
                file_path: file_path.display().to_string(),
                name: handler.to_string(),
            },
            confidence: ROUTE_CONFIDENCE,
            service: None,
            meta: vec![("method".into(), http_method.into())],
        });
    }
    out
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

/// Maps Spring annotation name → HTTP method string.
/// Returns `None` for non-route annotations (e.g. `PathVariable`, `RestController`).
fn http_method_from_annotation(ann: &str) -> Option<&'static str> {
    match ann {
        "GetMapping" => Some("GET"),
        "PostMapping" => Some("POST"),
        "PutMapping" => Some("PUT"),
        "DeleteMapping" => Some("DELETE"),
        "PatchMapping" => Some("PATCH"),
        "RequestMapping" => Some("ANY"),
        _ => None,
    }
}
