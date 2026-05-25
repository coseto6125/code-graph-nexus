//! Host-integration consistency. Report-only — `--fix` doesn't touch these
//! (host configs are user-owned; we surface drift, not auto-rewrite it).

use crate::admin::host_integration::{mcp, native};
use crate::admin::status::HostStatus;
use crate::commands::doctor::CheckResult;

pub(crate) fn check() -> Vec<CheckResult> {
    [
        ("host:claude-code", mcp::claude_code::status()),
        ("host:gemini-native", native::gemini::status()),
        ("host:codex-native", native::codex::status()),
    ]
    .into_iter()
    .map(|(name, status)| map_status(name, status))
    .collect()
}

/// A missing optional host is informational, not a failure — only an
/// `Outdated` config (stale relative to what ecp now writes) is a Warn.
fn map_status(name: &str, status: HostStatus) -> CheckResult {
    match status {
        HostStatus::Installed { detail } => CheckResult::ok(name, format!("installed ({detail})")),
        HostStatus::Outdated { reason } => CheckResult::warn(name, format!("outdated — {reason}"))
            .with_remediation(format!("ecp admin {} install", host_cli(name))),
        HostStatus::Missing => CheckResult::ok(name, "not integrated (optional)"),
    }
}

fn host_cli(name: &str) -> &str {
    match name {
        "host:claude-code" => "claude",
        "host:gemini-native" => "gemini",
        "host:codex-native" => "codex",
        _ => "claude",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::doctor::CheckStatus;

    #[test]
    fn installed_is_ok() {
        let r = map_status(
            "host:x",
            HostStatus::Installed {
                detail: "spawn".into(),
            },
        );
        assert_eq!(r.status, CheckStatus::Ok);
        assert!(r.message.contains("spawn"));
    }

    #[test]
    fn outdated_is_warn_with_remediation() {
        let r = map_status(
            "host:claude-code",
            HostStatus::Outdated {
                reason: "v1<v2".into(),
            },
        );
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.remediation.unwrap().contains("claude"));
    }

    #[test]
    fn missing_optional_host_is_ok_not_fail() {
        // A missing optional integration must not fail the whole doctor run.
        let r = map_status("host:codex-native", HostStatus::Missing);
        assert_eq!(r.status, CheckStatus::Ok);
    }
}
