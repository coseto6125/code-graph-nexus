//! `ecp dev pr-analyze` — classify a PR by area / risk / cross-PR semantic
//! conflict, emit JSON consumed by `.github/workflows/ecp-pr-analyze.yml`
//! to apply labels + commit statuses for Mergify routing.
//!
//! Black-box wraps `ecp impact --baseline <ref> --format json` (subprocess),
//! so no tight coupling to impact's internal API.

use clap::Args;
use ecp_core::EcpError;

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
    pub format: String,

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
