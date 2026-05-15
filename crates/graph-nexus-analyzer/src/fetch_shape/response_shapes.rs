//! Port of upstream `extractResponseShapes`
//! (`gitnexus/src/core/ingestion/route-extractors/response-shapes.ts`).
//!
//! Pure regex over a route handler's source. Detects response payload
//! emission and extracts top-level keys, classified by HTTP status:
//! - JS/TS: `res.status(N).json({...})` / `res.json({...})` /
//!   `return Response.json({...})` / `return new Response(JSON.stringify({...}))`
//! - PHP: `echo json_encode([...])` / `wp_send_json_*([...])`
//!
//! Status classification:
//! - `2xx` or status omitted → `response_keys` (success path)
//! - `4xx` or `5xx` → `error_keys` (error path)
//!
//! `shape_check` joins these against consumer accessed keys to find
//! drift. Returns empty vecs when no payload pattern matches.

/// Language hint for the extractor — controls which regex set runs.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Lang {
    JavaScript,
    TypeScript,
    Php,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ResponseShape {
    pub response_keys: Vec<String>,
    pub error_keys: Vec<String>,
}

/// SA-2 implementation goal: replicate upstream's extractor for
/// JS/TS first (PHP can be a follow-up if time permits — the lang
/// enum already accepts it). Both lists must be deduped before
/// return; order does not matter.
///
/// Tests should cover: `res.status(200).json({a,b})` →
/// `response_keys=[a,b]`; `res.status(400).json({error})` →
/// `error_keys=[error]`; `return Response.json({x})` →
/// `response_keys=[x]`; payload with shorthand props (`{id, name}`)
/// and explicit key:value pairs (`{id: u.id}`); nested objects
/// (only top-level keys captured); spread (`{...rest, id}`) preserves
/// `id`; non-matching files return `Default::default()`.
pub fn extract(_content: &str, _lang: Lang) -> ResponseShape {
    ResponseShape::default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_default() {
        let s = extract("", Lang::JavaScript);
        assert!(s.response_keys.is_empty());
        assert!(s.error_keys.is_empty());
    }
}
