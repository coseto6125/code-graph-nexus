//! Config / path sanity: ECP_HOME resolves and is writable; the registry
//! dir exists; the Claude skills parent dir exists.

use crate::commands::doctor::CheckResult;
use ecp_core::registry::resolve_home_ecp;
use std::path::{Path, PathBuf};

pub(crate) fn check() -> Vec<CheckResult> {
    let home_ecp = resolve_home_ecp();
    let mut out = vec![ecp_home_check(&home_ecp)];

    let claude_skills = claude_home().join("skills");
    out.push(if claude_skills.is_dir() {
        CheckResult::ok(
            "config:claude-dir",
            format!("{} exists", claude_skills.display()),
        )
    } else {
        CheckResult::warn(
            "config:claude-dir",
            format!(
                "{} missing — no skills installed yet",
                claude_skills.display()
            ),
        )
        .with_remediation("ecp admin claude install skills all")
    });

    out
}

/// ECP_HOME must resolve to an existing, writable dir. A write-probe failure
/// is a Warn (ecp falls back to a temp dir, degraded but functional), not Fail.
fn ecp_home_check(home_ecp: &Path) -> CheckResult {
    if !home_ecp.is_dir() {
        return CheckResult::warn(
            "config:ecp-home",
            format!(
                "{} does not exist yet (created on first index)",
                home_ecp.display()
            ),
        );
    }
    let probe = home_ecp.join(".doctor-write-probe");
    match std::fs::write(&probe, b"") {
        Ok(()) => {
            let _ = std::fs::remove_file(&probe);
            CheckResult::ok(
                "config:ecp-home",
                format!("{} writable", home_ecp.display()),
            )
        }
        Err(e) => CheckResult::warn(
            "config:ecp-home",
            format!(
                "{} not writable ({e}) — ecp will fall back to a temp dir",
                home_ecp.display()
            ),
        ),
    }
}

fn claude_home() -> PathBuf {
    std::env::var_os("HOME")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".claude")
}
