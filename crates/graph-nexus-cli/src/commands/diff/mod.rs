//! `gnx diff` — generalized graph-level cross-commit diff command.
//!
//! Compares two git refs and emits structural changes per requested section
//! (bindings / routes / contracts / all). See spec
//! `docs/superpowers/specs/2026-05-16-tier-b-surface-and-diff-design.md` §5.

use clap::{Args, ValueEnum};
use graph_nexus_core::GnxError;

pub mod baseline;
pub mod git_guard;

/// Section of the graph to diff. `All` = bindings + routes + contracts.
#[derive(ValueEnum, Clone, Debug, PartialEq, Eq, Hash)]
#[value(rename_all = "lowercase")]
pub enum DiffSection {
    Bindings,
    Routes,
    Contracts,
    All,
}

#[derive(Args, Debug, Clone)]
pub struct DiffArgs {
    /// Comma-separated section(s) to diff: bindings, routes, contracts, or all.
    #[arg(long, value_delimiter = ',', required = true)]
    pub section: Vec<DiffSection>,

    /// Git ref to compare against: branch / tag / commit SHA / HEAD~N / PR/<n>. No default.
    #[arg(long, required = true)]
    pub baseline: String,

    /// Output format: text (default) | json | toon
    #[arg(long, default_value = "text")]
    pub format: String,

    /// List every change (text format only). Default truncates to top-10 per section.
    #[arg(long, default_value_t = false)]
    pub verbose: bool,

    /// Repository root path (defaults to current directory).
    #[arg(long)]
    pub repo: Option<String>,
}

pub fn run(args: DiffArgs) -> Result<(), GnxError> {
    let repo_dir = match args.repo.as_deref() {
        Some(p) => std::path::PathBuf::from(p),
        None => std::env::current_dir()
            .map_err(|e| GnxError::Output(format!("cwd: {e}")))?,
    };

    let baseline_sha = baseline::resolve(&args.baseline, &repo_dir)?;
    let current_sha = baseline::resolve("HEAD", &repo_dir)?;

    let want_bindings = args
        .section
        .iter()
        .any(|s| matches!(s, DiffSection::Bindings | DiffSection::All));
    let want_routes = args
        .section
        .iter()
        .any(|s| matches!(s, DiffSection::Routes | DiffSection::All));
    let want_contracts = args
        .section
        .iter()
        .any(|s| matches!(s, DiffSection::Contracts | DiffSection::All));

    // Build baseline snapshot via temp checkout.
    let _baseline_data = {
        let _guard = git_guard::GitGuard::enter(&repo_dir, &baseline_sha)?;
        snapshot_sections(&repo_dir, &args.section)?
    }; // _guard dropped here, restores branch + stash

    let _current_data = snapshot_sections(&repo_dir, &args.section)?;
    let _ = (current_sha, want_bindings, want_routes, want_contracts);

    // Section diff lands in Tasks 9-12.
    Err(GnxError::Output(
        "section diff not yet implemented".into(),
    ))
}

fn snapshot_sections(
    _repo_dir: &std::path::Path,
    _sections: &[DiffSection],
) -> Result<SectionSnapshot, GnxError> {
    Ok(SectionSnapshot::default())
}

#[derive(Default)]
#[allow(dead_code)] // fields populated in Tasks 9-12
pub(crate) struct SectionSnapshot {
    pub bindings: Vec<serde_json::Value>,
    pub routes: Vec<serde_json::Value>,
    pub contracts: Vec<serde_json::Value>,
}
