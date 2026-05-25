//! `ecp doctor` — environment health check.
//!
//! Aggregates independent checks (installed skills freshness, graph index
//! freshness, host-integration consistency, config/path sanity) into one
//! report. Default is read-only; `--fix` reruns the fixable remediations
//! (skill reinstall, index rebuild) in place.

mod checks;

use clap::Args;
use ecp_core::EcpError;
use serde::Serialize;

#[derive(Args, Debug, Clone)]
pub struct DoctorArgs {
    /// Apply fixable remediations (reinstall stale skills, rebuild stale index).
    /// Host-integration and config findings are report-only.
    #[arg(long)]
    pub fix: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CheckStatus {
    Ok,
    Warn,
    Fail,
}

#[derive(Debug, Clone, Serialize)]
pub struct CheckResult {
    pub name: String,
    pub status: CheckStatus,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub remediation: Option<String>,
    /// Set only when `--fix` ran for this check: whether the fix succeeded.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub fix_applied: Option<bool>,
}

impl CheckResult {
    pub fn ok(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Ok,
            message: message.into(),
            remediation: None,
            fix_applied: None,
        }
    }

    pub fn warn(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Warn,
            message: message.into(),
            remediation: None,
            fix_applied: None,
        }
    }

    pub fn fail(name: impl Into<String>, message: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            status: CheckStatus::Fail,
            message: message.into(),
            remediation: None,
            fix_applied: None,
        }
    }

    pub fn with_remediation(mut self, hint: impl Into<String>) -> Self {
        self.remediation = Some(hint.into());
        self
    }
}

pub fn run(args: DoctorArgs) -> Result<(), EcpError> {
    let mut results = Vec::new();
    results.extend(checks::skills::check(args.fix));
    results.push(checks::index::check(args.fix));
    results.extend(checks::host::check());
    results.extend(checks::config::check());

    let mut warn = 0usize;
    let mut fail = 0usize;
    for r in &results {
        let tag = match r.status {
            CheckStatus::Ok => "ok  ",
            CheckStatus::Warn => {
                warn += 1;
                "warn"
            }
            CheckStatus::Fail => {
                fail += 1;
                "fail"
            }
        };
        println!("[{tag}] {}: {}", r.name, r.message);
        if let Some(hint) = &r.remediation {
            match r.fix_applied {
                Some(true) => println!("       fixed: ran `{hint}`"),
                Some(false) => println!("       fix failed — run manually: `{hint}`"),
                None => println!("       hint: {hint}"),
            }
        }
    }

    println!(
        "\n{} checks · {} ok · {} warn · {} fail",
        results.len(),
        results.len() - warn - fail,
        warn,
        fail
    );

    if fail > 0 {
        return Err(EcpError::Output(format!("doctor: {fail} check(s) failed")));
    }
    Ok(())
}
