//! Saga compensation pair detection: `CompensatedBy` edge emission.
//!
//! ## What this pass does
//!
//! Detects compensator function names following the Saga convention and pairs
//! them with their corresponding operation nodes, emitting
//! `RelType::CompensatedBy` edges (Task 3).
//!
//! A *compensator* is a function whose name starts with one of the roots
//! `compensate`, `undo`, or `rollback`, followed by an underscore separator
//! (snake_case) or a case boundary (camelCase / PascalCase), and then the
//! operation name suffix.
//!
//! ## Case handling
//!
//! | Input name        | Root    | Recovered operation |
//! |-------------------|---------|---------------------|
//! | `undo_book_room`  | `undo`  | `book_room`         |
//! | `undoBookRoom`    | `undo`  | `bookRoom`          |
//! | `UndoBookRoom`    | `Undo`  | `BookRoom`          |
//!
//! The recovered operation name preserves the suffix's **original case** so
//! that it matches the operation node's name verbatim in the graph.
//!
//! # Task 3 note
//! Task 3 will add `emit_edges` + the graph/pool imports to this file.

/// Compensator roots, lower-cased. Matched as a prefix on the lower-cased name.
const COMPENSATOR_ROOTS: &[&str] = &["compensate", "undo", "rollback"];

/// Result of stripping a compensator root: the bare operation name with its
/// ORIGINAL case preserved (so it matches the operation node's name).
#[derive(Debug, PartialEq, Eq)]
pub struct CompensatorMatch {
    pub operation_name: String,
}

/// If `name` is a compensator (`<root>` followed by a `_`-separator or a
/// case-boundary), return the bare operation name with original case. Else None.
pub fn strip_compensator_root(name: &str) -> Option<CompensatorMatch> {
    let lower = name.to_ascii_lowercase();
    for &root in COMPENSATOR_ROOTS {
        if !lower.starts_with(root) {
            continue;
        }
        let rest = &name[root.len()..];
        if rest.is_empty() {
            continue; // root with no suffix
        }
        // snake_case separator
        if let Some(suffix) = rest.strip_prefix('_') {
            if !suffix.is_empty() {
                return Some(CompensatorMatch {
                    operation_name: suffix.to_string(),
                });
            }
            continue;
        }
        // camel/Pascal boundary: next char must start a new word (uppercase).
        let first = rest.chars().next().unwrap();
        if first.is_ascii_uppercase() {
            let compensator_first = name.chars().next().unwrap();
            if compensator_first.is_ascii_uppercase() {
                // PascalCase: operation keeps its capital (BookRoom).
                return Some(CompensatorMatch {
                    operation_name: rest.to_string(),
                });
            }
            // camelCase: lowercase the operation's first char (bookRoom).
            let mut chars = rest.chars();
            let lowered: String = chars
                .next()
                .map(|c| c.to_ascii_lowercase())
                .into_iter()
                .chain(chars)
                .collect();
            return Some(CompensatorMatch {
                operation_name: lowered,
            });
        }
    }
    None
}
