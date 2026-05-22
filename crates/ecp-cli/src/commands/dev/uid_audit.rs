//! `ecp dev uid-audit` — cluster-collapsed view of `uid-collision` BlindSpot
//! records.
//!
//! **DEV-ONLY. NOT AN LLM SIGNAL.** This surface exists for ecp parser
//! maintainers tracking residual uid hash collisions after parser changes.
//! End-user / agent LLM consumption belongs in `ecp summary`.
//!
//! A single parser gap (e.g. missing `owner_class` on Go struct fields named
//! `File`) can fire thousands of distinct `BlindSpotRecord`s. The raw count
//! `uid-collision: N` hides the fact that those N records collapse into
//! 20-40 cluster identities. This command exposes the cluster view —
//! ranked by cluster size — so a parser developer can prioritise root-cause
//! fixes by impact rather than chasing one record at a time.
//!
//! Each cluster key is `(lang, second_kind, second_owner, second_name)`,
//! parsed from the BlindSpot's `hint` field (format
//! `"{bs_kind}: first={k}:{p}:{o}:{n} second={k}:{p}:{o}:{n}"`).

use crate::commit_lookup::find_latest_by_mtime;
use crate::output::{emit, OutputFormat};
use clap::Args;
use ecp_core::graph::ArchivedZeroCopyGraph;
use ecp_core::registry::{resolve_home_ecp, Registry};
use ecp_core::EcpError;
use memmap2::Mmap;
use serde_json::Value;
use std::collections::HashMap;
use std::fs::File;
use std::path::PathBuf;

#[derive(Args, Debug, Clone)]
pub struct UidAuditArgs {
    /// Repository selector (path | name | dir_name). Defaults to cwd-resolved
    /// repo. Picks the most-recently-built `graph.bin` under
    /// `~/.ecp/<dir_name>/commits/`.
    #[arg(long)]
    pub repo: Option<String>,

    /// Path to a specific `graph.bin` (bypasses registry resolution). Useful
    /// for auditing an arbitrary snapshot.
    #[arg(long)]
    pub graph: Option<PathBuf>,

    /// Maximum number of clusters to show (sorted by cluster size desc).
    #[arg(long, default_value_t = 40)]
    pub top: usize,

    /// Filter by `second_kind` (e.g. `Variable`, `Function`, `Method`).
    #[arg(long)]
    pub kind: Option<String>,

    /// Filter by derived language (e.g. `Python`, `Go`, `Java`).
    #[arg(long)]
    pub lang: Option<String>,

    /// Output format. Default `text` (table). `json` and `toon` available
    /// for downstream tooling.
    #[arg(long)]
    pub format: Option<String>,
}

pub fn run(args: UidAuditArgs) -> Result<(), EcpError> {
    let graph_path = resolve_graph_path(&args)?;
    let f = File::open(&graph_path)
        .map_err(|e| EcpError::InvalidArgument(format!("open {}: {e}", graph_path.display())))?;
    let mmap = unsafe {
        Mmap::map(&f)
            .map_err(|e| EcpError::InvalidArgument(format!("mmap {}: {e}", graph_path.display())))?
    };
    let graph = rkyv::access::<ArchivedZeroCopyGraph, rkyv::rancor::Error>(&mmap)
        .map_err(|e| EcpError::InvalidArgument(format!("rkyv access: {e}")))?;

    let report = build_report(graph, &args);

    let format = OutputFormat::parse(args.format.as_deref());
    match format {
        OutputFormat::Text => print_text(&report, &graph_path),
        _ => emit(
            &serde_json::to_value(&report).unwrap_or(Value::Null),
            format,
        )?,
    }
    Ok(())
}

fn resolve_graph_path(args: &UidAuditArgs) -> Result<PathBuf, EcpError> {
    if let Some(p) = &args.graph {
        return Ok(p.clone());
    }
    let home_ecp = resolve_home_ecp();
    let registry = Registry::open(&home_ecp)
        .map_err(|e| EcpError::InvalidArgument(format!("registry open: {e}")))?;
    let reg = registry.snapshot();
    let cwd = std::env::current_dir().unwrap_or_default();
    let sel = args.repo.as_deref().unwrap_or(".");
    let selector =
        crate::repo_selector::parse(sel).map_err(|e| EcpError::Output(format!("selector: {e}")))?;
    let cwd_str = cwd.to_string_lossy();
    let resolved =
        crate::repo_selector::resolve_top_level(&selector, reg, &cwd_str, "dev uid-audit")
            .map_err(|e| EcpError::Output(format!("selector: {e}")))?;
    let r = resolved
        .first()
        .ok_or_else(|| EcpError::InvalidArgument("no repo resolved from selector".into()))?;
    let commits_dir = home_ecp.join(&r.dir_name).join("commits");
    find_latest_by_mtime(&commits_dir)
        .map(|d| d.join("graph.bin"))
        .ok_or_else(|| {
            EcpError::InvalidArgument(format!(
                "no graph.bin under {} — run `ecp admin index` first",
                commits_dir.display()
            ))
        })
}

/// Map a file extension to its display language. Mirrors the dispatch in
/// `crates/ecp-analyzer/src/pipeline.rs` so cluster labels stay consistent
/// with the rest of ecp.
fn lang_from_path(p: &str) -> &'static str {
    let ext = p.rsplit('.').next().unwrap_or("");
    match ext {
        "ts" | "tsx" => "TypeScript",
        "js" | "jsx" | "mjs" | "cjs" => "JavaScript",
        "py" => "Python",
        "java" => "Java",
        "kt" | "kts" => "Kotlin",
        "cs" => "CSharp",
        "go" => "Go",
        "rs" => "Rust",
        "php" => "PHP",
        "rb" => "Ruby",
        "swift" => "Swift",
        "c" => "C",
        "h" | "cc" | "cpp" | "cxx" | "hpp" | "hxx" | "hh" => "C++",
        "dart" => "Dart",
        "sh" | "bash" => "Bash",
        "lua" | "luau" => "Lua",
        "vue" => "Vue",
        "svelte" => "Svelte",
        "yml" | "yaml" => "YAML",
        _ => "?",
    }
}

/// Parse `BlindSpotRecord.hint` of shape
/// `"<bs_kind>: first=K:P:O:N second=K:P:O:N"` and return the four `second=`
/// fields. Returns `None` if the format is unexpected — callers count these
/// as "unparsed" and surface the figure so silent parser drift is visible.
fn parse_hint(hint: &str) -> Option<(&str, &str, &str, &str)> {
    let second = hint.split(" second=").nth(1)?;
    let mut parts = second.splitn(4, ':');
    let kind = parts.next()?;
    let path = parts.next()?;
    let owner = parts.next()?;
    let name = parts.next()?;
    Some((kind, path, owner, name))
}

#[derive(serde::Serialize)]
struct Report {
    /// Total `uid-collision` records scanned (pre-filter).
    total: u32,
    /// Records whose `hint` couldn't be parsed into the
    /// `first=…/second=…` shape — silent parser drift if non-zero.
    hint_unparsed: u32,
    /// Distinct `(lang, second_kind, owner, name)` cluster identities
    /// after filters applied.
    distinct_clusters: usize,
    /// Top-N clusters by size (descending).
    top: Vec<Cluster>,
    /// Fraction (0..1) of total records covered by `top`.
    top_coverage: f64,
}

#[derive(serde::Serialize)]
struct Cluster {
    count: u32,
    lang: String,
    second_kind: String,
    owner_class: String,
    name: String,
    sample_path: String,
}

fn build_report(graph: &ArchivedZeroCopyGraph, args: &UidAuditArgs) -> Report {
    let mut clusters: HashMap<(String, String, String, String), (u32, String)> = HashMap::new();
    let mut total_uid_collision: u32 = 0;
    let mut total_hint_unparsed: u32 = 0;

    for bs in graph.blind_spots.iter() {
        let kind = bs.kind.resolve(&graph.string_pool);
        if kind != "uid-collision" {
            continue;
        }
        total_uid_collision += 1;
        let hint = bs.hint.resolve(&graph.string_pool);
        let Some((second_kind, second_path, second_owner, second_name)) = parse_hint(hint) else {
            total_hint_unparsed += 1;
            continue;
        };

        // Apply filters AFTER parsing — that way the "unparsed" count is
        // honest even when filters are narrow.
        if let Some(want_kind) = args.kind.as_deref() {
            if second_kind != want_kind {
                continue;
            }
        }
        let lang = lang_from_path(second_path);
        if let Some(want_lang) = args.lang.as_deref() {
            if !lang.eq_ignore_ascii_case(want_lang) {
                continue;
            }
        }

        let key = (
            lang.to_string(),
            second_kind.to_string(),
            second_owner.to_string(),
            second_name.to_string(),
        );
        clusters
            .entry(key)
            .and_modify(|(c, _)| *c += 1)
            .or_insert((1, second_path.to_string()));
    }

    let distinct = clusters.len();
    let mut rows: Vec<((String, String, String, String), (u32, String))> =
        clusters.into_iter().collect();
    rows.sort_by(|a, b| b.1 .0.cmp(&a.1 .0));

    let top: Vec<Cluster> = rows
        .iter()
        .take(args.top)
        .map(|((lang, kind, owner, name), (count, sample))| Cluster {
            count: *count,
            lang: lang.clone(),
            second_kind: kind.clone(),
            owner_class: owner.clone(),
            name: name.clone(),
            sample_path: sample.clone(),
        })
        .collect();

    let covered: u32 = top.iter().map(|c| c.count).sum();
    let top_coverage = if total_uid_collision > 0 {
        covered as f64 / total_uid_collision as f64
    } else {
        0.0
    };

    Report {
        total: total_uid_collision,
        hint_unparsed: total_hint_unparsed,
        distinct_clusters: distinct,
        top,
        top_coverage,
    }
}

fn print_text(report: &Report, graph_path: &std::path::Path) {
    // Warning header: keep the dev-only nature loud — this output is NOT
    // for LLM agents (the kinds shown here are parser hash-collision
    // aggregates, not source opacity).
    eprintln!("┌─ ecp dev uid-audit ─────────────────────────────────────────┐");
    eprintln!("│ DEV-ONLY · NOT an LLM signal · for ecp parser maintainers   │");
    eprintln!("│ For source-code opacity / LLM-actionable blind spots, run:  │");
    eprintln!("│   ecp summary --repo .                                      │");
    eprintln!("└─────────────────────────────────────────────────────────────┘");
    println!("graph                        : {}", graph_path.display());
    println!("total uid-collision records  : {}", report.total);
    println!(
        "distinct (lang,kind,own,name): {}",
        report.distinct_clusters
    );
    println!("hint parse failures          : {}", report.hint_unparsed);
    println!();
    println!(
        "{:>5} {:<12} {:<14} {:<28} {:<28} {}",
        "count", "lang", "kind", "owner_class", "name", "sample_path"
    );
    println!("{}", "-".repeat(120));
    for c in &report.top {
        let owner_disp = if c.owner_class.is_empty() {
            "(none)"
        } else {
            c.owner_class.as_str()
        };
        let sample_short = if c.sample_path.len() > 50 {
            format!("...{}", &c.sample_path[c.sample_path.len() - 47..])
        } else {
            c.sample_path.clone()
        };
        println!(
            "{:>5} {:<12} {:<14} {:<28} {:<28} {}",
            c.count, c.lang, c.second_kind, owner_disp, c.name, sample_short
        );
    }
    println!();
    println!(
        "top {} clusters cover {} / {} ({:.1}%)",
        report.top.len(),
        report.top.iter().map(|c| c.count).sum::<u32>(),
        report.total,
        100.0 * report.top_coverage
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parse_hint_extracts_second_fields() {
        let hint =
            "uid-collision: first=Variable:src/a.py:Outer:File second=Variable:src/b.py:Inner:File";
        let got = parse_hint(hint).expect("hint must parse");
        assert_eq!(got, ("Variable", "src/b.py", "Inner", "File"));
    }

    #[test]
    fn parse_hint_missing_second_returns_none() {
        assert!(parse_hint("uid-collision: first=Variable:src/a.py:Outer:File").is_none());
    }

    #[test]
    fn parse_hint_empty_owner_is_kept_as_empty() {
        let hint = "uid-collision: first=Variable:src/a.py::File second=Function:src/b.go::main";
        let got = parse_hint(hint).expect("hint must parse");
        assert_eq!(got.2, ""); // owner_class can be empty (top-level)
        assert_eq!(got.3, "main");
    }

    #[test]
    fn lang_from_path_known_extensions() {
        assert_eq!(lang_from_path("src/x.py"), "Python");
        assert_eq!(lang_from_path("src/x.rs"), "Rust");
        assert_eq!(lang_from_path("src/x.go"), "Go");
        assert_eq!(lang_from_path("src/x.h"), "C++");
        assert_eq!(lang_from_path("src/x.c"), "C");
        assert_eq!(lang_from_path("noext"), "?");
    }
}
