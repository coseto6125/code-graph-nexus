//! `ecp dev pr-analyze` — classify a PR by area / risk / cross-PR semantic
//! conflict, emit JSON consumed by `.github/workflows/ecp-pr-analyze.yml`
//! to apply labels + commit statuses for Mergify routing.
//!
//! Black-box wraps `ecp impact --baseline <ref> --format json` (subprocess),
//! so no tight coupling to impact's internal API.

use crate::output::OutputFormat;
use clap::Args;
use ecp_core::EcpError;
use serde::Serialize;

#[derive(Serialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum Area {
    Parser,
    Cli,
    Test,
    Docs,
}

#[derive(Serialize, Debug, PartialEq, Eq, Clone, Copy)]
#[serde(rename_all = "kebab-case")]
pub enum Risk {
    Low,
    Medium,
    High,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct CrossPrConflict {
    pub pr: u32,
    pub overlap_symbols: Vec<String>,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct StatusSuggestion {
    pub context: String,
    pub state: String, // "success" | "pending"
    pub description: String,
}

#[derive(Serialize, Debug, PartialEq, Eq)]
pub struct PrAnalyzeOutput {
    pub pr_number: u32,
    pub head_sha: String,
    pub baseline_sha: String,
    pub area: Option<Area>,
    pub risk: Risk,
    pub impact_size: usize,
    pub changed_symbols: Vec<String>,
    pub cross_pr_conflicts: Vec<CrossPrConflict>,
    pub suggested_labels: Vec<String>,
    pub suggested_status: StatusSuggestion,
}

#[derive(Args, Debug, Clone)]
pub struct PrAnalyzeArgs {
    /// Base ref to diff against (typically `origin/main`).
    #[arg(long)]
    pub baseline: String,

    /// PR head ref (typically `HEAD` inside the PR-checkout workflow).
    #[arg(long = "pr-head")]
    pub pr_head: String,

    /// PR number — required to look up sibling PRs via gh CLI.
    #[arg(long = "pr-number")]
    pub pr_number: u32,

    /// Label scoping the cross-PR conflict scan. Defaults to `merge-queue`.
    #[arg(long = "queue-label", default_value = "merge-queue")]
    pub queue_label: String,

    /// Output format. Workflow consumes JSON.
    #[arg(long, default_value = "json")]
    pub format: OutputFormat,

    /// Do not write/update own cache comment, do not call gh mutations.
    #[arg(long = "dry-run")]
    pub dry_run: bool,
}

pub fn run(args: PrAnalyzeArgs, _cli_graph: &std::path::Path) -> Result<(), EcpError> {
    // Stub — implemented incrementally in later tasks.
    let _ = args;
    eprintln!("pr-analyze: not yet implemented");
    Ok(())
}
