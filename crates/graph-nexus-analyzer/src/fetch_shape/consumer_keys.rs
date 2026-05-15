//! Port of upstream `extractConsumerAccessedKeys`
//! (`gitnexus/src/core/ingestion/call-processor.ts:3283-3327`).
//!
//! Pure regex over file content. Detects three access patterns on
//! HTTP response variables and returns the union of accessed keys,
//! filtered by [`RESPONSE_ACCESS_BLOCKLIST`] to drop common JS API /
//! Array / Promise / DOM method names that would otherwise produce
//! false positives.
//!
//! Patterns (matches upstream exactly):
//! 1. Destructuring from `.json()` chain — `const {a,b} = await res.json()`
//!    also `const {a} = await (await fetch(...)).json()`
//! 2. Destructuring from a `data|result|response|json|body|res` variable
//!    — `const {a,b} = data`
//! 3. Property access on the same variable name list —
//!    `data.foo`, `response?.bar`, `result.baz`
//!
//! Returns a deduped `Vec<String>`. Empty when no patterns match.

/// SA-1 implementation goal: replicate upstream's three regex passes.
/// The function signature is fixed (`&str -> Vec<String>`). Order of
/// returned keys does not matter — shape_check builds a set.
///
/// Tests should cover: each of the three patterns in isolation, the
/// blocklist filter (e.g., `data.length` must not appear), aliased
/// destructuring (`{a: aliasA}` keeps `a`, not `aliasA`), optional
/// chaining (`data?.foo`), and the empty-input case.
pub fn extract(_content: &str) -> Vec<String> {
    Vec::new()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_input_returns_empty() {
        assert!(extract("").is_empty());
    }
}
