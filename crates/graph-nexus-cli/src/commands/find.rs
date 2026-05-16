//! `gnx find <name>` — single-definition lookup by symbol name.
//!
//! Returns the single most-likely definition for a named symbol.
//! Fills the gap between `gnx search` (ranked top-K, concept search)
//! and `gnx inspect` (full context, edges, impact).
//!
//! Ranking when multiple nodes share the same name:
//! 1. Category priority: Source=0, Document=1, Config=2, Test=3
//! 2. caller_count desc (real definitions usually have more callers)
//! 3. File path alphabetical (deterministic tie-break)
//!
//! Test files are skipped by default; `--include-tests` overrides.

use crate::commands::format::kind_to_str;
use crate::engine::Engine;
use crate::output::{emit, OutputFormat};
use clap::Args;
use graph_nexus_core::graph::{ArchivedFileCategory, ArchivedZeroCopyGraph};
use graph_nexus_core::GnxError;

#[derive(Args, Debug, Clone)]
pub struct FindArgs {
    /// Symbol name to find. Default is exact match; use `--fuzzy` for substring.
    pub pattern: String,

    /// Substring / fuzzy match instead of exact.
    #[arg(long)]
    pub fuzzy: bool,

    /// Return all matching definitions instead of just the top-1.
    #[arg(long)]
    pub all: bool,

    /// Filter by node kinds (csv: function,method,class,...).
    #[arg(long)]
    pub kind: Option<String>,

    /// Repository selector. Same semantics as `gnx search --repo`.
    #[arg(long)]
    pub repo: Option<String>,

    /// Output format: text (default) | json | toon.
    #[arg(long)]
    pub format: Option<String>,

    /// Include hits from test files (default skipped).
    #[arg(long)]
    pub include_tests: bool,
}

#[derive(Debug, serde::Serialize)]
pub struct FindMatch {
    pub file: String,
    pub line: u32,
    pub name: String,
    pub kind: String,
    pub category: String,
    pub caller_count: u32,
    pub signature: String,
}

#[derive(Debug, serde::Serialize)]
pub struct FindResult {
    pub found: bool,
    pub matches: Vec<FindMatch>,
    pub status: String,
}

/// Category sort priority: lower = higher priority.
fn category_priority(cat: &ArchivedFileCategory) -> u8 {
    match cat {
        ArchivedFileCategory::Source => 0,
        ArchivedFileCategory::Document => 1,
        ArchivedFileCategory::Config => 2,
        ArchivedFileCategory::Test => 3,
    }
}

fn category_to_str(cat: &ArchivedFileCategory) -> &'static str {
    match cat {
        ArchivedFileCategory::Source => "Source",
        ArchivedFileCategory::Test => "Test",
        ArchivedFileCategory::Document => "Document",
        ArchivedFileCategory::Config => "Config",
    }
}

/// Count incoming edges for a node (its callers/importers).
fn count_callers(graph: &ArchivedZeroCopyGraph, node_idx: usize) -> u32 {
    let in_start = graph.in_offsets[node_idx].to_native() as usize;
    let in_end = graph.in_offsets[node_idx + 1].to_native() as usize;
    (in_end - in_start) as u32
}

pub fn run(args: FindArgs, engine: &Engine) -> Result<(), GnxError> {
    let graph = engine.graph().map_err(|e| GnxError::Rkyv(e.to_string()))?;
    let format = OutputFormat::parse(args.format.as_deref());

    let kind_filter: Option<Vec<String>> = args.kind.as_deref().map(|s| {
        s.split(',')
            .map(|p| p.trim().to_ascii_lowercase())
            .filter(|p| !p.is_empty())
            .collect()
    });

    // Collect all candidate nodes matching the name pattern.
    let mut candidates: Vec<(usize, u32, u8, String)> = graph
        .nodes
        .iter()
        .enumerate()
        .filter_map(|(node_idx, node)| {
            let name = node.name.resolve(&graph.string_pool);
            let matches = if args.fuzzy {
                name.contains(args.pattern.as_str())
            } else {
                name == args.pattern.as_str()
            };
            if !matches {
                return None;
            }

            // Kind filter
            if let Some(ref kinds) = kind_filter {
                let node_kind = kind_to_str(&node.kind).to_ascii_lowercase();
                if !kinds.iter().any(|k| k == &node_kind) {
                    return None;
                }
            }

            let file = &graph.files[node.file_idx.to_native() as usize];
            let file_path = file.path.resolve(&graph.string_pool);

            // Skip test files unless --include-tests
            if !args.include_tests && matches!(file.category, ArchivedFileCategory::Test) {
                return None;
            }

            let prio = category_priority(&file.category);
            let caller_count = count_callers(graph, node_idx);

            Some((node_idx, caller_count, prio, file_path.to_string()))
        })
        .collect();

    // Sort: category priority asc, caller_count desc, file path asc
    candidates.sort_unstable_by(|a, b| {
        a.2.cmp(&b.2)
            .then_with(|| b.1.cmp(&a.1))
            .then_with(|| a.3.cmp(&b.3))
    });

    let selected: Vec<_> = if args.all {
        candidates
    } else {
        candidates.into_iter().take(1).collect()
    };

    let matches: Vec<FindMatch> = selected
        .into_iter()
        .map(|(node_idx, caller_count, _, _)| {
            let node = &graph.nodes[node_idx];
            let file = &graph.files[node.file_idx.to_native() as usize];
            FindMatch {
                file: file.path.resolve(&graph.string_pool).to_string(),
                line: node.span.0.to_native(),
                name: node.name.resolve(&graph.string_pool).to_string(),
                kind: kind_to_str(&node.kind).to_string(),
                category: category_to_str(&file.category).to_string(),
                caller_count,
                signature: node.uid.resolve(&graph.string_pool).to_string(),
            }
        })
        .collect();

    let found = !matches.is_empty();

    match format {
        OutputFormat::Text => {
            if !found {
                println!("no match for: {}", args.pattern);
                return Ok(());
            }
            for m in &matches {
                let test_tag = if m.category == "Test" { " [test]" } else { "" };
                println!(
                    "[{}] {}:{}{} ({}) callers={}",
                    m.kind, m.file, m.line, test_tag, m.name, m.caller_count
                );
            }
            Ok(())
        }
        _ => {
            let result = FindResult {
                found,
                matches,
                status: "success".to_string(),
            };
            emit(
                &serde_json::to_value(&result).map_err(|e| GnxError::Output(e.to_string()))?,
                format,
            )
        }
    }
}
