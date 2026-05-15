//! UserPromptSubmit handler: surface async reindex outcomes via marker
//! files, then unlink them so each event fires only once. Failure takes
//! priority over success because it is more actionable.

use super::common::{emit_additional_context, gitnexus_dir, HookInput};
use graph_nexus_core::GnxError;
use std::fs;
use std::path::Path;

pub fn handle(input: &HookInput) -> Result<(), GnxError> {
    let gnx_dir = match gitnexus_dir(&input.cwd) {
        Some(d) => d,
        None => return Ok(()),
    };
    let complete = gnx_dir.join(".rebuild-complete");
    let failed = gnx_dir.join(".rebuild-failed");
    let log = gnx_dir.join("last-rebuild.log");

    if failed.exists() {
        let tail = read_log_tail(&log, 3);
        let _ = fs::remove_file(&failed);
        let msg = format!(
            "gnx background reindex FAILED. {} Run `gnx admin index` manually to retry.",
            if tail.is_empty() {
                String::new()
            } else {
                format!("Last log lines: {tail}.")
            }
        );
        emit_additional_context("UserPromptSubmit", msg.trim());
        return Ok(());
    }

    if complete.exists() {
        let stats = read_stats(&gnx_dir);
        let _ = fs::remove_file(&complete);
        let msg = format!(
            "gnx index rebuild complete ({stats}). gnx tools now return fresh data."
        );
        emit_additional_context("UserPromptSubmit", &msg);
    }
    Ok(())
}

fn read_log_tail(log: &Path, lines: usize) -> String {
    let raw = match fs::read_to_string(log) {
        Ok(s) => s,
        Err(_) => return String::new(),
    };
    let collected: Vec<&str> = raw.trim().lines().rev().take(lines).collect();
    collected.into_iter().rev().collect::<Vec<_>>().join(" | ")
}

fn read_stats(gnx_dir: &Path) -> String {
    let raw = match fs::read_to_string(gnx_dir.join("meta.json")) {
        Ok(s) => s,
        Err(_) => return "?".into(),
    };
    let v: serde_json::Value = match serde_json::from_str(&raw) {
        Ok(v) => v,
        Err(_) => return "?".into(),
    };
    let nodes = v
        .get("node_count")
        .and_then(|x| x.as_u64())
        .map(|n| n.to_string())
        .unwrap_or_else(|| "?".into());
    let edges = v
        .get("edge_count")
        .and_then(|x| x.as_u64())
        .map(|n| n.to_string())
        .unwrap_or_else(|| "?".into());
    format!("{nodes} symbols, {edges} rels")
}
