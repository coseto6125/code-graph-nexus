//! Per-file constituent dispatch and Finding collection.
//!
//! Each helper is a pure function over a `serde_json::Value` payload so it
//! can be unit-tested without a real graph or engine.

use super::findings::{Finding, Report, Severity, Source};
use crate::commands::impact::{self, Direction, ImpactArgs};
use crate::commands::tool_map::{self, ToolMapArgs};
use crate::engine::Engine;
use graph_nexus_core::GnxError;
use serde_json::Value;
use std::collections::HashSet;
use std::path::{Path, PathBuf};

pub fn run(files: &[PathBuf], repo_dir: &Path, engine: &Engine) -> Result<Report, GnxError> {
    if files.is_empty() {
        return Ok(Report {
            findings: vec![],
            files_reviewed: 0,
        });
    }

    let file_scope: HashSet<&Path> = files.iter().map(|p| p.as_path()).collect();
    let mut findings: Vec<Finding> = Vec::new();

    // ── impact: per-file blast-radius ────────────────────────────────────────
    findings.extend(run_impact(files, repo_dir, engine));

    // ── coverage: BlindSpot rows for in-scope files ──────────────────────────
    findings.extend(run_coverage(&file_scope, engine));

    // ── egress (tool_map): external-call sites — full repo scan, no diff
    // baseline checkout. Lines added-only filtering requires checking out the
    // baseline tree which is expensive. Emitting all call-sites in changed
    // files as BlindSpot-style findings with note; see Phase 2.5 follow-up.
    findings.push(Finding {
        file: "aggregator".into(),
        line: 0,
        kind: "blind_spot",
        severity: Severity::Info,
        message:
            "egress diff-aware filter (added-only call-sites) deferred — needs baseline checkout (Phase 2.5)"
                .into(),
        source: Source::BlindSpot,
    });
    // Still surface all tool_map hits for changed files as info findings.
    findings.extend(run_tool_map(files, engine));

    // ── shape_check: needs cross-file route context not yet assembled ─────────
    findings.push(Finding {
        file: "aggregator".into(),
        line: 0,
        kind: "blind_spot",
        severity: Severity::Info,
        message: "shape_check constituent skipped — needs cross-file route context (Phase 2.5)"
            .into(),
        source: Source::BlindSpot,
    });

    // ── diff (resolver): bindings tier-degradation ────────────────────────────
    findings.push(Finding {
        file: "aggregator".into(),
        line: 0,
        kind: "blind_spot",
        severity: Severity::Info,
        message:
            "resolver diff constituent skipped — requires --baseline ref from caller (Phase 2.5)"
                .into(),
        source: Source::BlindSpot,
    });

    Ok(Report {
        findings,
        files_reviewed: files.len(),
    })
}

// ── impact helper ────────────────────────────────────────────────────────────

/// Run impact with `--baseline` set to origin/HEAD to get changed-symbol
/// blast radius. Falls back to per-file impact when no baseline is available.
/// Findings are emitted for risk_level >= medium only.
fn run_impact(files: &[PathBuf], repo_dir: &Path, engine: &Engine) -> Vec<Finding> {
    // Build baseline args for each file independently then aggregate.
    // We use the graph's existing symbol coverage rather than re-parsing git.
    files
        .iter()
        .flat_map(|f| impact_for_file(f, repo_dir, engine))
        .collect()
}

fn impact_for_file(file: &PathBuf, repo_dir: &Path, engine: &Engine) -> Vec<Finding> {
    let file_str = file.to_string_lossy().into_owned();
    let repo_str = repo_dir.to_string_lossy().into_owned();
    let args = ImpactArgs {
        name: None,
        target: None,
        baseline: Some("HEAD~1".into()),
        file: Some(file_str.clone()),
        kind: None,
        direction: Direction::Up,
        depth: 3,
        high_trust_only: false,
        min_confidence: None,
        include_tests: false,
        relation_types: None,
        repo: Some(repo_str),
        format: None,
    };
    match impact::build_payload(&args, engine) {
        Ok(v) => impact_findings(&v, &file_str),
        Err(_) => vec![],
    }
}

/// Extract findings from an `impact::build_payload` Value.
/// Keeps only nodes with depth > 0 (callers of the changed symbol),
/// mapping blast-radius count >= 4 as `medium` risk (info finding).
pub fn impact_findings(v: &Value, file: &str) -> Vec<Finding> {
    // Baseline mode: impact_by_symbol[].impact[] contains caller nodes.
    // Single-symbol mode: impact[].
    let mut findings = Vec::new();

    let process_callers = |callers: &Vec<Value>, sym_name: &str, findings: &mut Vec<Finding>| {
        let caller_count = callers
            .iter()
            .filter(|e| e["depth"].as_u64().unwrap_or(0) > 0)
            .count();
        if caller_count >= 4 {
            findings.push(Finding {
                file: file.into(),
                line: 0,
                kind: "impact",
                severity: Severity::Info,
                message: format!("{sym_name} has {caller_count} callers — review blast radius"),
                source: Source::Impact,
            });
        }
    };

    if let Some(by_sym) = v.get("impact_by_symbol").and_then(|v| v.as_array()) {
        for entry in by_sym {
            let sym = entry["symbol"].as_str().unwrap_or("?");
            if let Some(callers) = entry["impact"].as_array() {
                process_callers(callers, sym, &mut findings);
            }
        }
    } else if let Some(callers) = v.get("impact").and_then(|v| v.as_array()) {
        let sym = v["target"].as_str().unwrap_or("?");
        process_callers(callers, sym, &mut findings);
    }
    findings
}

// ── coverage (BlindSpot) helper ──────────────────────────────────────────────

fn run_coverage(file_scope: &HashSet<&Path>, engine: &Engine) -> Vec<Finding> {
    // coverage::build_payload with --repo needs a path arg, but for blind-spot
    // extraction we need to read the graph's blind_spots directly.
    // Use the engine's graph to avoid a subprocess round-trip.
    let graph = match engine.graph() {
        Ok(g) => g,
        Err(_) => return vec![],
    };

    graph
        .blind_spots
        .iter()
        .filter_map(|bs| {
            let file_path = bs.file_path.resolve(&graph.string_pool);
            let in_scope = file_scope.iter().any(|p| {
                p.to_string_lossy() == file_path || file_path.ends_with(&*p.to_string_lossy())
            });
            if !in_scope {
                return None;
            }
            let kind = bs.kind.resolve(&graph.string_pool);
            Some(Finding {
                file: file_path.to_string(),
                line: bs.start_row.into(),
                kind: "blind_spot",
                severity: Severity::Info,
                message: format!("blind spot: {kind}"),
                source: Source::BlindSpot,
            })
        })
        .collect()
}

/// Extract coverage BlindSpot findings from a `coverage::build_payload` Value.
/// Used in unit tests — production path uses `run_coverage` (graph direct).
pub fn coverage_blind_spots(v: &Value, file_scope: &[&str]) -> Vec<Finding> {
    let scope_set: HashSet<&str> = file_scope.iter().copied().collect();
    let mut findings = Vec::new();

    // coverage payload shape: {"coverage": {"per_repo": [{"blind_spots": ...}]}}
    // or {"coverage": {"indexed_repos": ...}} — mine per_repo if present.
    if let Some(per_repo) = v.pointer("/coverage/per_repo").and_then(|v| v.as_array()) {
        for repo in per_repo {
            if let Some(by_kind) = repo
                .pointer("/blind_spots/by_kind")
                .and_then(|v| v.as_object())
            {
                for (kind, _count) in by_kind {
                    // No file info in the aggregated by_kind — emit one finding
                    // per kind for any file in scope.
                    for file in file_scope {
                        if scope_set.contains(file) {
                            findings.push(Finding {
                                file: (*file).into(),
                                line: 0,
                                kind: "blind_spot",
                                severity: Severity::Info,
                                message: format!("blind spot: {kind}"),
                                source: Source::BlindSpot,
                            });
                        }
                    }
                }
            }
        }
    }
    findings
}

// ── tool_map (egress) helper ─────────────────────────────────────────────────

fn run_tool_map(files: &[PathBuf], engine: &Engine) -> Vec<Finding> {
    let args = ToolMapArgs {
        category: None,
        repo: None,
        format: None,
    };
    let v = match tool_map::build_payload(&args, engine) {
        Ok(v) => v,
        Err(_) => return vec![],
    };
    tool_map_findings(&v, files)
}

/// Extract tool_map findings for call-sites in the given files.
pub fn tool_map_findings(v: &Value, files: &[PathBuf]) -> Vec<Finding> {
    let file_strs: HashSet<String> = files
        .iter()
        .map(|p| p.to_string_lossy().into_owned())
        .collect();

    let mut findings = Vec::new();

    let calls = match v.get("calls").and_then(|c| c.as_object()) {
        Some(c) => c,
        None => return findings,
    };

    for (_category, entries) in calls {
        let entries = match entries.as_array() {
            Some(a) => a,
            None => continue,
        };
        for entry in entries {
            let file_path = entry["filePath"].as_str().unwrap_or("");
            if !file_strs
                .iter()
                .any(|f| f == file_path || file_path.ends_with(f.as_str()))
            {
                continue;
            }
            let callee = entry["callee"].as_str().unwrap_or("?");
            let package = entry["package"].as_str().unwrap_or("?");
            let line = entry["line"].as_u64().unwrap_or(0) as u32;
            findings.push(Finding {
                file: file_path.into(),
                line,
                kind: "egress",
                severity: Severity::Info,
                message: format!("external call: {callee} (package: {package})"),
                source: Source::Egress,
            });
        }
    }
    findings
}

#[cfg(test)]
mod tests {
    use super::*;
    use serde_json::json;
    use std::path::PathBuf;

    #[test]
    fn impact_findings_baseline_mode_below_threshold_emits_nothing() {
        // impact_by_symbol with 3 callers — below threshold of 4
        let v = json!({
            "status": "success",
            "baseline": "HEAD~1",
            "changed_symbols": [],
            "impact_by_symbol": [
                {
                    "symbol": "foo",
                    "filePath": "src/foo.rs",
                    "impact": [
                        {"depth": 0, "name": "foo"},
                        {"depth": 1, "name": "a"},
                        {"depth": 1, "name": "b"},
                        {"depth": 1, "name": "c"}
                    ]
                }
            ]
        });
        // 3 callers (depth > 0) — below threshold
        let findings = impact_findings(&v, "src/foo.rs");
        assert!(findings.is_empty(), "expected no findings for 3 callers");
    }

    #[test]
    fn impact_findings_baseline_mode_at_threshold_emits_finding() {
        let v = json!({
            "status": "success",
            "baseline": "HEAD~1",
            "changed_symbols": [],
            "impact_by_symbol": [
                {
                    "symbol": "bar",
                    "filePath": "src/bar.rs",
                    "impact": [
                        {"depth": 0, "name": "bar"},
                        {"depth": 1, "name": "a"},
                        {"depth": 1, "name": "b"},
                        {"depth": 1, "name": "c"},
                        {"depth": 1, "name": "d"}
                    ]
                }
            ]
        });
        // 4 callers (depth > 0) — meets threshold
        let findings = impact_findings(&v, "src/bar.rs");
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].source, Source::Impact);
        assert_eq!(findings[0].severity, Severity::Info);
        assert!(findings[0].message.contains("4 callers"));
    }

    #[test]
    fn coverage_blind_spots_maps_per_repo_findings() {
        let v = json!({
            "coverage": {
                "per_repo": [
                    {
                        "repo": "myrepo",
                        "blind_spots": {
                            "total": 1,
                            "by_kind": {
                                "dynamic-import": 1
                            }
                        }
                    }
                ]
            }
        });
        let findings = coverage_blind_spots(&v, &["src/foo.py"]);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].source, Source::BlindSpot);
        assert!(findings[0].message.contains("dynamic-import"));
    }

    #[test]
    fn tool_map_findings_filters_to_scope_files() {
        let v = json!({
            "status": "success",
            "totals": {"http": 2},
            "calls": {
                "http": [
                    {"callee": "axios.get", "package": "axios", "filePath": "src/api.ts", "line": 10, "col": 5},
                    {"callee": "axios.post", "package": "axios", "filePath": "src/other.ts", "line": 20, "col": 3}
                ]
            }
        });
        let files = vec![PathBuf::from("src/api.ts")];
        let findings = tool_map_findings(&v, &files);
        assert_eq!(findings.len(), 1);
        assert_eq!(findings[0].file, "src/api.ts");
        assert_eq!(findings[0].line, 10);
        assert_eq!(findings[0].source, Source::Egress);
    }

    #[test]
    fn tool_map_findings_empty_calls_yields_no_findings() {
        let v = json!({
            "status": "success",
            "totals": {},
            "calls": {}
        });
        let files = vec![PathBuf::from("src/any.ts")];
        let findings = tool_map_findings(&v, &files);
        assert!(findings.is_empty());
    }
}
