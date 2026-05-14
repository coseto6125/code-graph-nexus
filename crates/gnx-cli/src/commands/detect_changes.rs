//! `gnx detect_changes` — list symbols touched by git diff hunks and assess
//! the blast radius via affected Process execution-flows.
//!
//! Algorithm (mirrors upstream `local-backend.ts:detectChanges`):
//!   1. Run `git diff -U0` for the requested scope
//!   2. Parse hunks into per-file line ranges
//!   3. For each (file, hunk), find graph nodes whose span overlaps the hunk
//!   4. For each touched symbol, look up which Process traces contain it
//!   5. Risk = bucket on affected-process count (0 / 1-5 / 6-15 / >15)

use crate::commands::format::kind_to_str;
use crate::engine::Engine;
use crate::git::{DiffScope, GitDiffProvider, ShellGitProvider};
use crate::output::{emit, OutputFormat};
use clap::Args;
use gnx_core::graph::ArchivedNodeKind;
use gnx_core::algorithms::process_trace::is_test_path;
use gnx_core::graph_query::{file_idx_by_suffix, nodes_overlapping_lines, processes_containing};
use gnx_core::GnxError;
use serde_json::json;
use std::collections::HashMap;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct DetectChangesArgs {
    /// Diff scope: `unstaged` (default), `staged`, `all`, or `compare`.
    #[arg(long, default_value = "unstaged")]
    pub scope: String,

    /// Required when `--scope compare`: the ref to diff against (e.g. `HEAD~1`).
    #[arg(long)]
    pub base_ref: Option<String>,

    /// Path to the repo root (defaults to current directory).
    #[arg(long)]
    pub repo: Option<String>,

    /// Output format: `toon` (default), `json`, or `text`.
    #[arg(long, default_value = "toon")]
    pub format: Option<String>,

    /// Filter changes to specific NodeKinds (comma-separated, e.g.
    /// `function,method,class`). When omitted, all kinds are reported.
    #[arg(long)]
    pub kind: Option<String>,

    /// Include test-file hunks (default: false — test files dropped).
    #[arg(long, default_value_t = false)]
    pub include_tests: bool,
}

pub fn run(args: DetectChangesArgs, engine: &Engine) -> Result<(), GnxError> {
    let repo_path = PathBuf::from(args.repo.as_deref().unwrap_or("."));
    let scope = DiffScope::parse(Some(&args.scope), args.base_ref.as_deref())?;
    let format = OutputFormat::parse(args.format.as_deref());

    let provider = ShellGitProvider;
    let file_diffs = provider.diff(&repo_path, &scope)?;

    if file_diffs.is_empty() {
        let result = json!({
            "summary": {
                "changed_count": 0,
                "affected_count": 0,
                "risk_level": "none",
                "message": "No changes detected."
            },
            "changed_symbols": [],
            "affected_processes": [],
        });
        return emit(&result, format);
    }

    let graph = engine.graph().map_err(|e| GnxError::Rkyv(e.to_string()))?;

    let kind_filter = parse_kind_filter(args.kind.as_deref());

    // Map diff hunks → changed symbols
    let mut changed_symbols = Vec::new();
    let mut changed_node_indices: Vec<u32> = Vec::new();
    let mut changed_files_counted: usize = 0;

    for fd in &file_diffs {
        if !args.include_tests && is_test_path(&fd.file_path) {
            continue;
        }
        changed_files_counted += 1;

        let Some(file_idx) = file_idx_by_suffix(graph, &fd.file_path) else {
            continue;
        };

        for hunk in &fd.hunks {
            // git diff line numbers are 1-based; graph stores spans 0-based.
            let hunk_start = hunk.start_line.saturating_sub(1);
            let hunk_end = hunk.end_line.saturating_sub(1);
            let overlap = nodes_overlapping_lines(graph, file_idx, hunk_start, hunk_end);
            for (idx, node) in overlap {
                if !kind_matches(&node.kind, &kind_filter) {
                    continue;
                }
                // Skip File and Process nodes — File always overlaps every hunk,
                // Process is a virtual aggregate not a source-level symbol.
                if matches!(node.kind, ArchivedNodeKind::File | ArchivedNodeKind::Process) {
                    continue;
                }
                if changed_node_indices.contains(&idx) {
                    continue; // dedup across hunks within one file
                }
                changed_node_indices.push(idx);

                let file_node = &graph.files[node.file_idx.to_native() as usize];
                changed_symbols.push(json!({
                    "id": node.uid.resolve(&graph.string_pool),
                    "name": node.name.resolve(&graph.string_pool),
                    "type": kind_to_str(&node.kind),
                    "filePath": file_node.path.resolve(&graph.string_pool),
                    "line": node.span.0.to_native(),
                    "change_type": "touched",
                }));
            }
        }
    }

    // Find affected processes
    let process_start = graph.process_start.to_native();
    let mut affected: HashMap<u32, AffectedProcess> = HashMap::new();
    for &node_idx in &changed_node_indices {
        for (proc_idx, step) in processes_containing(graph, node_idx) {
            let proc_node = &graph.nodes[proc_idx as usize];
            let entry = affected.entry(proc_idx).or_insert_with(|| {
                let k = (proc_idx - process_start) as usize;
                let off_s = graph.traces_offsets[k].to_native() as usize;
                let off_e = graph.traces_offsets[k + 1].to_native() as usize;
                let trace = &graph.traces_data[off_s..off_e];
                let step_count = trace.len() as u32;
                // Derive process_type from communities the trace touches.
                let mut comms: Vec<u16> = trace
                    .iter()
                    .map(|x| graph.nodes[x.to_native() as usize].community_id.to_native())
                    .filter(|&c| c != 0)
                    .collect();
                comms.sort_unstable();
                comms.dedup();
                let process_type = if comms.len() > 1 {
                    "cross_community"
                } else {
                    "intra_community"
                };
                AffectedProcess {
                    id: proc_node.uid.resolve(&graph.string_pool).to_string(),
                    name: proc_node.name.resolve(&graph.string_pool).to_string(),
                    process_type,
                    step_count,
                    changed_steps: Vec::new(),
                }
            });
            entry.changed_steps.push((
                graph.nodes[node_idx as usize]
                    .name
                    .resolve(&graph.string_pool)
                    .to_string(),
                step,
            ));
        }
    }

    let process_count = affected.len();
    let risk_level = match process_count {
        0 => "low",
        1..=5 => "medium",
        6..=15 => "high",
        _ => "critical",
    };

    let affected_arr: Vec<_> = affected
        .into_values()
        .map(|p| {
            json!({
                "id": p.id,
                "name": p.name,
                "process_type": p.process_type,
                "step_count": p.step_count,
                "changed_steps": p.changed_steps
                    .iter()
                    .map(|(s, step)| json!({ "symbol": s, "step": step }))
                    .collect::<Vec<_>>(),
            })
        })
        .collect();

    let result = json!({
        "summary": {
            "changed_count": changed_symbols.len(),
            "affected_count": process_count,
            "changed_files": changed_files_counted,
            "risk_level": risk_level,
        },
        "changed_symbols": changed_symbols,
        "affected_processes": affected_arr,
    });

    emit(&result, format)
}

struct AffectedProcess {
    id: String,
    name: String,
    process_type: &'static str,
    step_count: u32,
    changed_steps: Vec<(String, u32)>,
}

fn parse_kind_filter(s: Option<&str>) -> Option<Vec<String>> {
    s.map(|raw| {
        raw.split(',')
            .map(|p| p.trim().to_ascii_lowercase())
            .filter(|p| !p.is_empty())
            .collect()
    })
}

fn kind_matches(kind: &ArchivedNodeKind, filter: &Option<Vec<String>>) -> bool {
    let Some(f) = filter else {
        return true;
    };
    let s = kind_to_str(kind).to_ascii_lowercase();
    f.iter().any(|k| k == &s)
}
