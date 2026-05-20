//! Hard time + byte caps on a single tree-sitter parse.
//!
//! `Parser::parse(source, None)` has no built-in cancellation. A pathological
//! input (huge generated source, regex-heavy strings, deeply nested templates
//! after error recovery) can pin a rayon worker for seconds. The progress
//! callback API added in tree-sitter v0.25.0 lets the parser bail at the next
//! checkpoint, which is what `parse_with_budget` wires up here.
//!
//! Callers should use [`ParseBudget::DEFAULT`] unless they have a reason to
//! tighten the limits — the defaults (1s wall-clock, 8 MiB byte advance) only
//! fire on inputs the indexer should be skipping anyway.

use std::ops::ControlFlow;
use std::time::{Duration, Instant};
use tree_sitter::{ParseOptions, Parser, Tree};

/// Cap a single parse on wall-clock duration and how far into the source
/// the parser is allowed to walk. Reaching either limit aborts the parse;
/// `parse_with_budget` then returns `None`.
#[derive(Clone, Copy, Debug)]
pub struct ParseBudget {
    pub max_duration: Duration,
    pub max_bytes: usize,
}

impl ParseBudget {
    pub const DEFAULT: Self = Self {
        max_duration: Duration::from_secs(1),
        max_bytes: 8 * 1024 * 1024,
    };
}

impl Default for ParseBudget {
    fn default() -> Self {
        Self::DEFAULT
    }
}

/// Drop-in replacement for `Parser::parse(source, None)` with a hard budget.
///
/// Returns `None` when the budget is exhausted or the parser otherwise fails
/// (same failure shape as the unbudgeted form). Callers map `None` to the
/// existing "parse failed" error path.
pub fn parse_with_budget(parser: &mut Parser, source: &[u8], budget: ParseBudget) -> Option<Tree> {
    let start = Instant::now();
    let len = source.len();
    let mut callback = |state: &tree_sitter::ParseState| -> ControlFlow<()> {
        if state.current_byte_offset() > budget.max_bytes || start.elapsed() > budget.max_duration {
            ControlFlow::Break(())
        } else {
            ControlFlow::Continue(())
        }
    };
    let options = ParseOptions::new().progress_callback(&mut callback);
    parser.parse_with_options(
        &mut |i, _| if i < len { &source[i..] } else { &[] },
        None,
        Some(options),
    )
}
