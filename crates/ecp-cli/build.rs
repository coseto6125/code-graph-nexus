//! Build script for `ecp-cli`.
//!
//! Embeds the current git short-SHA into the binary at compile time as
//! `ECP_GIT_SHA`. The runtime reads it via `env!()` / `option_env!()` to:
//!   * Display the version string (`ecp --version`) as `<semver>+<sha>`.
//!   * Persist the SHA into `CommitBuildMeta.binary_commit_sha` so callers
//!     can detect when the graph was built by a different binary revision.

fn main() {
    println!("cargo:rerun-if-changed=../../.git/HEAD");
    println!("cargo:rerun-if-changed=../../.git/refs/heads");

    let sha = std::process::Command::new("git")
        .args(["rev-parse", "--short", "HEAD"])
        .output()
        .ok()
        .and_then(|o| {
            if o.status.success() {
                String::from_utf8(o.stdout).ok()
            } else {
                None
            }
        })
        .map(|s| s.trim().to_string())
        .filter(|s| !s.is_empty())
        .unwrap_or_else(|| "unknown".to_string());

    println!("cargo:rustc-env=ECP_GIT_SHA={sha}");
}
