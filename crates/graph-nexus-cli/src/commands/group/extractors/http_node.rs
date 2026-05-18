//! Node/TypeScript HTTP route extractor: Express + Fastify patterns via tree-sitter.

use crate::commands::group::types::{
    ContractRole, ContractType, ExtractedContract, SymbolRef,
};
use std::path::Path;
use streaming_iterator::StreamingIterator;
use tree_sitter::{Parser, Query, QueryCursor};

pub(super) const ROUTE_CONFIDENCE: f32 = 0.85;

/// Matches `app.get("/path", handler)` / `router.post(...)` etc.
/// `string_fragment` is already unquoted in tree-sitter-typescript.
const QUERY_SRC: &str = r#"
(call_expression
  function: (member_expression
    property: (property_identifier) @method)
  arguments: (arguments
    (string
      (string_fragment) @path)
    (_) @handler))
"#;

pub fn extract_http(file_path: &Path, source: &[u8]) -> Vec<ExtractedContract> {
    let mut parser = Parser::new();
    let lang: tree_sitter::Language = tree_sitter_typescript::LANGUAGE_TYPESCRIPT.into();
    if let Err(e) = parser.set_language(&lang) {
        tracing::warn!("group::extract_http (node): set_language failed: {e:?}");
        return Vec::new();
    }
    let Some(tree) = parser.parse(source, None) else {
        tracing::warn!(
            "group::extract_http (node): parser.parse returned None for {}",
            file_path.display()
        );
        return Vec::new();
    };
    let query = match Query::new(&lang, QUERY_SRC) {
        Ok(q) => q,
        Err(e) => {
            tracing::warn!("group::extract_http (node): Query::new failed: {e:?}");
            return Vec::new();
        }
    };

    let method_idx = match query.capture_index_for_name("method") {
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
        let method_name = capture_text(m, method_idx, source);
        let Some(http_method) = http_method_from_property(method_name) else {
            continue;
        };
        let route_path = capture_text(m, path_idx, source);
        // handler capture is the third arg — could be arrow function, identifier, etc.
        // Use its text as a symbol hint; for arrow functions this is verbose but honest.
        let handler_text = capture_text(m, handler_idx, source);
        let handler_name = first_identifier(handler_text);
        let id = format!("http:{http_method}:{route_path}");
        out.push(ExtractedContract {
            contract_id: id,
            contract_type: ContractType::Http,
            role: ContractRole::Provider,
            symbol_uid: format!("{}::{handler_name}", file_path.display()),
            symbol_ref: SymbolRef {
                file_path: file_path.display().to_string(),
                name: handler_name.to_string(),
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

/// Maps Express/Fastify method property name → HTTP method string.
/// Returns `None` for non-route properties (e.g. `listen`, `use`, `json`, ...).
fn http_method_from_property(name: &str) -> Option<&'static str> {
    match name {
        "get" => Some("GET"),
        "post" => Some("POST"),
        "put" => Some("PUT"),
        "delete" => Some("DELETE"),
        "patch" => Some("PATCH"),
        "all" => Some("ANY"),
        _ => None,
    }
}

/// Best-effort: extract leading identifier from handler text (e.g. `myHandler` from
/// `myHandler` or `<unknown>` from an arrow function literal).
fn first_identifier(text: &str) -> &str {
    let trimmed = text.trim();
    // If it starts with a letter/underscore, take the identifier prefix.
    let end = trimmed
        .find(|c: char| !c.is_alphanumeric() && c != '_')
        .unwrap_or(trimmed.len());
    if end > 0 {
        &trimmed[..end]
    } else {
        "<unknown>"
    }
}
