//! Integration test for `ecp impact --baseline --format json` `changed_paths`
//! envelope field (FU-2026-05-23-044).
//!
//! Verifies that the un-filtered `git diff --name-only` list is exposed on
//! the JSON envelope so downstream consumers like `ecp dev pr-analyze` can
//! do area classification without spawning a second `git diff` subprocess.
//!
//! Three scenarios:
//!   1. Pure docs-only diff (README.md whitespace change) — `changed_paths`
//!      contains the doc, `changed_symbols` is empty.
//!   2. Mixed code + docs diff — both files appear in `changed_paths`.
//!   3. Pure code diff — `changed_paths` matches the touched code file.

use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn ecp_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ecp")
}

fn run_git(repo: &Path, args: &[&str]) {
    let out = Command::new("git")
        .args(args)
        .current_dir(repo)
        .output()
        .expect("git failed to spawn");
    assert!(
        out.status.success(),
        "git {args:?} failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn git_commit(repo: &Path, msg: &str) {
    run_git(repo, &["add", "-A"]);
    run_git(
        repo,
        &[
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "-q",
            "-m",
            msg,
        ],
    );
}

fn ecp_index(repo: &Path) {
    let out = Command::new(ecp_bin())
        .args(["admin", "index", "--repo", "."])
        .current_dir(repo)
        .env("HOME", repo)
        .output()
        .expect("admin index spawn failed");
    assert!(
        out.status.success(),
        "admin index failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

fn run_impact_baseline(repo: &Path) -> Value {
    let out = Command::new(ecp_bin())
        .args([
            "impact",
            "--baseline",
            "HEAD~1",
            "--repo",
            ".",
            "--format",
            "json",
        ])
        .current_dir(repo)
        .env("HOME", repo)
        .output()
        .expect("impact --baseline failed to spawn");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        out.status.success(),
        "impact --baseline failed\nstderr={stderr}\nstdout={stdout}"
    );
    let json_start = stdout
        .find('{')
        .unwrap_or_else(|| panic!("no JSON in stdout:\n{stdout}\nstderr={stderr}"));
    serde_json::from_str(&stdout[json_start..])
        .unwrap_or_else(|e| panic!("JSON parse failed: {e}\nstdout={stdout}"))
}

const CODE_V1: &str = r#"
export function helper(): number {
    return 1;
}
"#;

const CODE_V2: &str = r#"
export function helper(): number {
    return 2;
}
"#;

/// Docs-only diff. `changed_symbols` is empty (no semantic change), but
/// `changed_paths` must still carry `README.md` so pr-analyze can classify
/// the PR as `Area::Docs` instead of falling through to the default queue.
#[test]
fn changed_paths_includes_docs_only_diff() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    std::fs::create_dir_all(repo.join("src")).unwrap();

    run_git(repo, &["init", "-q", "-b", "main"]);
    // Tests point HOME=repo so each test has its own isolated `.ecp/` cache;
    // gitignore it so cache files don't pollute the `git diff` we assert on.
    std::fs::write(repo.join(".gitignore"), ".ecp/\n").unwrap();
    std::fs::write(repo.join("src/lib.ts"), CODE_V1).unwrap();
    std::fs::write(repo.join("README.md"), "# Project\n").unwrap();
    git_commit(repo, "init");
    ecp_index(repo);

    // Docs-only second commit.
    std::fs::write(repo.join("README.md"), "# Project\n\nNew section.\n").unwrap();
    git_commit(repo, "docs: add section");
    ecp_index(repo);

    let val = run_impact_baseline(repo);
    let paths = val["changed_paths"]
        .as_array()
        .unwrap_or_else(|| panic!("`changed_paths` missing or not array:\n{val}"));
    let path_strs: Vec<&str> = paths.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        path_strs.contains(&"README.md"),
        "docs-only diff: README.md should be in changed_paths; got {path_strs:?}"
    );

    // changed_symbols must be empty (no semantic change).
    let symbols = val["changed_symbols"]
        .as_array()
        .unwrap_or_else(|| panic!("`changed_symbols` missing or not array:\n{val}"));
    assert!(
        symbols.is_empty(),
        "docs-only diff should produce 0 changed_symbols; got {symbols:?}"
    );
}

/// Mixed code + docs diff — both files must appear in `changed_paths`.
#[test]
fn changed_paths_includes_mixed_code_and_docs() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    std::fs::create_dir_all(repo.join("src")).unwrap();

    run_git(repo, &["init", "-q", "-b", "main"]);
    // Tests point HOME=repo so each test has its own isolated `.ecp/` cache;
    // gitignore it so cache files don't pollute the `git diff` we assert on.
    std::fs::write(repo.join(".gitignore"), ".ecp/\n").unwrap();
    std::fs::write(repo.join("src/lib.ts"), CODE_V1).unwrap();
    std::fs::write(repo.join("README.md"), "# Project\n").unwrap();
    git_commit(repo, "init");
    ecp_index(repo);

    std::fs::write(repo.join("src/lib.ts"), CODE_V2).unwrap();
    std::fs::write(repo.join("README.md"), "# Project\n\nUpdated.\n").unwrap();
    git_commit(repo, "tweak helper + docs");
    ecp_index(repo);

    let val = run_impact_baseline(repo);
    let paths = val["changed_paths"]
        .as_array()
        .unwrap_or_else(|| panic!("`changed_paths` missing:\n{val}"));
    let path_strs: Vec<&str> = paths.iter().filter_map(|v| v.as_str()).collect();
    assert!(
        path_strs.contains(&"src/lib.ts"),
        "mixed diff: src/lib.ts missing from changed_paths; got {path_strs:?}"
    );
    assert!(
        path_strs.contains(&"README.md"),
        "mixed diff: README.md missing from changed_paths; got {path_strs:?}"
    );
}

/// Pure code diff — `changed_paths` should still be populated (single source
/// of truth for area classification; pr-analyze relies on this).
#[test]
fn changed_paths_includes_pure_code_diff() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    std::fs::create_dir_all(repo.join("src")).unwrap();

    run_git(repo, &["init", "-q", "-b", "main"]);
    // Tests point HOME=repo so each test has its own isolated `.ecp/` cache;
    // gitignore it so cache files don't pollute the `git diff` we assert on.
    std::fs::write(repo.join(".gitignore"), ".ecp/\n").unwrap();
    std::fs::write(repo.join("src/lib.ts"), CODE_V1).unwrap();
    git_commit(repo, "init");
    ecp_index(repo);

    std::fs::write(repo.join("src/lib.ts"), CODE_V2).unwrap();
    git_commit(repo, "tweak helper");
    ecp_index(repo);

    let val = run_impact_baseline(repo);
    let paths = val["changed_paths"]
        .as_array()
        .unwrap_or_else(|| panic!("`changed_paths` missing:\n{val}"));
    let path_strs: Vec<&str> = paths.iter().filter_map(|v| v.as_str()).collect();
    assert_eq!(
        path_strs,
        vec!["src/lib.ts"],
        "pure code diff: expected exactly [src/lib.ts] in changed_paths"
    );
}
