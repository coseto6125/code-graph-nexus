//! Registry / index-store health: orphan dirs, missing graphs, corrupt meta.
//! Reuses the same `registry_health` scan the `ecp admin` diagnostics TUI runs.

use crate::admin::diagnostics::{registry_health, RegistryHealth};
use crate::commands::admin::doctor::CheckResult;
use ecp_core::registry::resolve_home_ecp;

pub(crate) fn check(fix: bool) -> Vec<CheckResult> {
    let home = resolve_home_ecp();
    let health = match registry_health(&home) {
        Ok(h) => h,
        Err(e) => return vec![CheckResult::fail("registry", format!("scan failed: {e}"))],
    };

    // Orphan index dirs are the one safely-removable category: their top-level
    // repo dir isn't in the registry, so nothing references them. Retire via the
    // same `retire_dir_async` primitive `admin prune` uses (atomic rename +
    // background delete) rather than a bare remove_dir_all — one retire path for
    // the whole tool. Done before classify so fix_applied reflects the outcome.
    let orphan_fix = fix.then(|| {
        let removed = health
            .orphan_index_dirs
            .iter()
            .filter(|p| ecp_core::registry::retire_dir_async(p).is_ok())
            .count();
        removed == health.orphan_index_dirs.len()
    });

    classify(&health, orphan_fix)
}

/// Map a health scan to per-category findings. Pure (no fs) so it's unit-tested
/// against hand-built `RegistryHealth` values. `orphan_fix` carries the outcome
/// of the orphan deletion the caller already performed (None = not attempted).
fn classify(health: &RegistryHealth, orphan_fix: Option<bool>) -> Vec<CheckResult> {
    let mut out = Vec::new();

    if !health.orphan_index_dirs.is_empty() {
        let n = health.orphan_index_dirs.len();
        let mut r = CheckResult::warn("registry:orphans", format!("{n} orphan index dir(s)"))
            .with_remediation("ecp admin doctor registry --fix");
        r.fix_applied = orphan_fix;
        out.push(r);
    }

    // Missing graph/meta need a rebuild — report-only (doctor has no repo
    // context to rebuild a specific commit's graph).
    if !health.missing_graphs.is_empty() {
        out.push(
            CheckResult::warn(
                "registry:graphs",
                format!("{} missing graph.bin", health.missing_graphs.len()),
            )
            .with_remediation("ecp admin index --repo <path>"),
        );
    }
    if !health.missing_meta.is_empty() {
        out.push(
            CheckResult::warn(
                "registry:meta",
                format!("{} missing meta.json", health.missing_meta.len()),
            )
            .with_remediation("ecp admin index --repo <path>"),
        );
    }
    // Corrupt meta is never auto-deleted (destructive on user data); a rebuild
    // overwrites it cleanly.
    if !health.corrupt_meta.is_empty() {
        out.push(
            CheckResult::warn(
                "registry:corrupt-meta",
                format!("{} corrupt meta.json", health.corrupt_meta.len()),
            )
            .with_remediation("ecp admin index --repo <path> (rebuild overwrites)"),
        );
    }

    if out.is_empty() {
        out.push(CheckResult::ok(
            "registry",
            format!("{} repo(s), no orphans or corruption", health.repo_count),
        ));
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::commands::admin::doctor::CheckStatus;
    use std::path::PathBuf;

    fn find<'a>(rs: &'a [CheckResult], name: &str) -> Option<&'a CheckResult> {
        rs.iter().find(|r| r.name == name)
    }

    #[test]
    fn clean_health_yields_single_ok() {
        let h = RegistryHealth {
            repo_count: 3,
            ..Default::default()
        };
        let out = classify(&h, None);
        assert_eq!(out.len(), 1);
        assert_eq!(out[0].status, CheckStatus::Ok);
    }

    #[test]
    fn orphans_warn_and_carry_fix_outcome() {
        let h = RegistryHealth {
            orphan_index_dirs: vec![PathBuf::from("/x/a"), PathBuf::from("/x/b")],
            ..Default::default()
        };
        // not fixed
        let out = classify(&h, None);
        let r = find(&out, "registry:orphans").unwrap();
        assert_eq!(r.status, CheckStatus::Warn);
        assert!(r.message.contains('2'));
        assert_eq!(r.fix_applied, None);
        // fixed
        let out = classify(&h, Some(true));
        assert_eq!(
            find(&out, "registry:orphans").unwrap().fix_applied,
            Some(true)
        );
    }

    #[test]
    fn missing_and_corrupt_are_report_only_never_fixed() {
        let h = RegistryHealth {
            missing_graphs: vec![PathBuf::from("/x/g")],
            corrupt_meta: vec![PathBuf::from("/x/m")],
            ..Default::default()
        };
        // even with orphan_fix Some, missing/corrupt carry no fix_applied
        let out = classify(&h, Some(true));
        assert_eq!(find(&out, "registry:graphs").unwrap().fix_applied, None);
        assert_eq!(
            find(&out, "registry:corrupt-meta").unwrap().fix_applied,
            None
        );
        assert!(find(&out, "registry").is_none()); // not "all clean"
    }
}
