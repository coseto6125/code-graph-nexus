//! PreToolUse handler: extract a search pattern from Grep / Glob / Bash
//! invocations, run an in-process gnx search, and inject the top-K
//! hits into the conversation as `additionalContext`. Capped at 5 hits
//! or ~2 KB serialized to keep the token cost bounded.

use super::common::{emit_additional_context, gitnexus_dir, strip_shell_quotes, HookInput};
use crate::commands::search::{compute_hits, Hit, SearchArgs, SearchMode};
use crate::engine::Engine;
use graph_nexus_core::GnxError;

const MAX_HITS: usize = 5;
const MAX_BYTES: usize = 2048;

pub fn handle(input: &HookInput) -> Result<(), GnxError> {
    let pattern = match extract_pattern(&input.tool_name, &input.tool_input) {
        Some(p) if p.len() >= 3 => p,
        _ => return Ok(()),
    };
    let gnx_dir = match gitnexus_dir(&input.cwd) {
        Some(d) => d,
        None => return Ok(()),
    };
    let graph_path = gnx_dir.join("graph.bin");
    let engine = match Engine::load(&graph_path) {
        Ok(e) => e,
        Err(_) => return Ok(()),
    };
    let args = SearchArgs {
        pattern,
        mode: SearchMode::Auto,
        kind: None,
        repo: None,
        format: None,
    };
    let hits = match compute_hits(args, &engine) {
        Ok(h) => h,
        Err(_) => return Ok(()),
    };
    if hits.is_empty() {
        return Ok(());
    }
    let lines = format_hits(&hits);
    if lines.trim().is_empty() {
        return Ok(());
    }
    emit_additional_context("PreToolUse", &lines);
    Ok(())
}

fn format_hits(hits: &[Hit]) -> String {
    let mut out = String::from("gnx graph hits:\n");
    let mut count = 0usize;
    for h in hits.iter().take(MAX_HITS) {
        let line = format!(
            "  [{}] {}:{} {} (callers:{}) score:{:.3}\n",
            h.kind, h.file, h.line, h.name, h.caller_count, h.score
        );
        if out.len() + line.len() > MAX_BYTES {
            break;
        }
        out.push_str(&line);
        count += 1;
    }
    if count == 0 {
        return String::new();
    }
    out
}

fn extract_pattern(tool: &str, tool_input: &serde_json::Value) -> Option<String> {
    match tool {
        "Grep" => tool_input
            .get("pattern")
            .and_then(|v| v.as_str())
            .map(str::to_string),
        "Glob" => {
            let raw = tool_input
                .get("pattern")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let re = regex::Regex::new(r"[*/]([a-zA-Z][a-zA-Z0-9_-]{2,})").ok()?;
            re.captures(raw).map(|c| c[1].to_string())
        }
        "Bash" => {
            let cmd = tool_input
                .get("command")
                .and_then(|v| v.as_str())
                .unwrap_or("");
            let stripped = strip_shell_quotes(cmd);
            extract_from_shell(&stripped)
        }
        _ => None,
    }
}

fn extract_from_shell(cmd: &str) -> Option<String> {
    let has_rg_or_grep = cmd.split_whitespace().any(|t| t == "rg" || t == "grep");
    if !has_rg_or_grep {
        return None;
    }
    let flags_with_values = [
        "-e",
        "-f",
        "-m",
        "-A",
        "-B",
        "-C",
        "-g",
        "--glob",
        "-t",
        "--type",
        "--include",
        "--exclude",
    ];
    let mut found_cmd = false;
    let mut skip_next = false;
    for token in cmd.split_whitespace() {
        if skip_next {
            skip_next = false;
            continue;
        }
        if !found_cmd {
            if token == "rg" || token == "grep" {
                found_cmd = true;
            }
            continue;
        }
        if token.starts_with('-') {
            if flags_with_values.contains(&token) {
                skip_next = true;
            }
            continue;
        }
        let cleaned: String = token.chars().filter(|c| *c != '"' && *c != '\'').collect();
        if cleaned.len() >= 3 {
            return Some(cleaned);
        }
    }
    None
}
