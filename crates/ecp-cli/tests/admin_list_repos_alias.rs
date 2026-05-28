//! Pins the `ecp admin list-repos` ↔ `ecp summary` alias contract.
//!
//! `list-repos` is a narrowed alias of `summary` (registry overview only): it
//! must produce the exact same stdout as `ecp summary` with no `--repo`, and
//! must reject `--repo` / `--detailed` so it cannot silently start meaning
//! "per-repo health" later.

use std::process::Command;

mod common;
use common::ecp_bin;

/// The alias must emit identical bytes to `ecp summary` (no `--repo`).
/// Same HOME / cwd → same registry view → same payload.
#[test]
fn list_repos_matches_summary_byte_for_byte() {
    let tmp = tempfile::tempdir().unwrap();

    let summary = Command::new(ecp_bin())
        .args(["summary", "--format", "json"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("ecp summary failed to spawn");
    assert!(
        summary.status.success(),
        "ecp summary failed: stderr={}",
        String::from_utf8_lossy(&summary.stderr)
    );

    let alias = Command::new(ecp_bin())
        .args(["admin", "list-repos", "--format", "json"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("ecp admin list-repos failed to spawn");
    assert!(
        alias.status.success(),
        "ecp admin list-repos failed: stderr={}",
        String::from_utf8_lossy(&alias.stderr)
    );

    assert_eq!(
        alias.stdout,
        summary.stdout,
        "list-repos stdout diverged from summary:\nalias={}\nsummary={}",
        String::from_utf8_lossy(&alias.stdout),
        String::from_utf8_lossy(&summary.stdout),
    );
}

/// `list-repos` must reject `--repo`: the whole point of the narrowed alias
/// is "registry overview, no per-repo drill-down". Accepting `--repo` would
/// let the verb drift back into being a full `summary` clone.
#[test]
fn list_repos_rejects_repo_flag() {
    let tmp = tempfile::tempdir().unwrap();

    let out = Command::new(ecp_bin())
        .args(["admin", "list-repos", "--repo", "anything"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("ecp admin list-repos failed to spawn");

    assert!(
        !out.status.success(),
        "expected --repo to be rejected, but command succeeded; stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
}

/// Same rationale for `--detailed`: the registry overview is intentionally
/// not detailed-mode-tunable through this alias.
#[test]
fn list_repos_rejects_detailed_flag() {
    let tmp = tempfile::tempdir().unwrap();

    let out = Command::new(ecp_bin())
        .args(["admin", "list-repos", "--detailed"])
        .current_dir(tmp.path())
        .env("HOME", tmp.path())
        .output()
        .expect("ecp admin list-repos failed to spawn");

    assert!(
        !out.status.success(),
        "expected --detailed to be rejected, but command succeeded; stdout={}",
        String::from_utf8_lossy(&out.stdout)
    );
}
