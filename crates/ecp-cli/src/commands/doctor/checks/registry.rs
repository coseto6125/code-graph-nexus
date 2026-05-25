//! Registry / index-store health: orphan dirs, missing graphs, corrupt meta.
//! Reuses the same `registry_health` scan the `ecp admin` diagnostics TUI runs.

use crate::admin::diagnostics::registry_health;
use crate::commands::doctor::CheckResult;
use ecp_core::registry::resolve_home_ecp;

pub(crate) fn check(fix: bool) -> Vec<CheckResult> {
    let home = resolve_home_ecp();
    let health = match registry_health(&home) {
        Ok(h) => h,
        Err(e) => return vec![CheckResult::fail("registry", format!("scan failed: {e}"))],
    };

    let mut out = Vec::new();

    // Orphan index dirs are the one safely-fixable category: they're not
    // referenced by the registry, so removing them only reclaims space.
    if !health.orphan_index_dirs.is_empty() {
        let n = health.orphan_index_dirs.len();
        let mut r = CheckResult::warn("registry:orphans", format!("{n} orphan index dir(s)"))
            .with_remediation("ecp doctor registry --fix");
        if fix {
            let removed = health
                .orphan_index_dirs
                .iter()
                .filter(|p| std::fs::remove_dir_all(p).is_ok())
                .count();
            r.fix_applied = Some(removed == n);
        }
        out.push(r);
    }

    // Missing graphs / meta need a rebuild — report-only (doctor has no repo
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
    // Corrupt meta is never auto-deleted (it's a destructive op on user data);
    // a rebuild overwrites it cleanly.
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
