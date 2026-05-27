//! Best-effort jsonl telemetry writer for MCP call_tool dispatch.
//!
//! Each MCP tool invocation appends one line to
//! `~/.ecp/telemetry/<repo>/calls.jsonl` where `<repo>` is the canonical
//! `repo_dir_name_for_cwd` key (basename + xxh3 hash). The MCP server
//! process MUST call [`init_repo_id`] at startup to register the key —
//! otherwise telemetry silently no-ops (the writer can't safely pick a
//! key on its own because `repo_dir_name_for_cwd` lives in `ecp-cli` and
//! depends on `git_cache`).
//!
//! Design constraints:
//! - Write path is best-effort: all I/O errors are silently dropped.
//! - Never blocks or panics the MCP dispatch path.
//! - Single cached `BufWriter` per process (one open/create_dir at boot;
//!   then one `writeln!` + `flush` per call, instead of open+write+close).
//! - Schema is **unstable (v1)** — appending new optional fields in v2 is
//!   backward-compatible (old readers ignore unknown keys).

use std::fs::{File, OpenOptions};
use std::io::{BufWriter, Write as _};
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};

/// Re-export so existing `crate::telemetry::rfc3339_now()` call sites
/// (server.rs::call_tool) keep working without importing `ecp_core::time`.
pub use ecp_core::time::rfc3339_now;

pub use ecp_core::telemetry::CallRecord;

/// Per-process repo identity, set by `ecp-cli` at MCP server boot via
/// `init_repo_id`. Format: `<basename>__<xxh3_hash>` (from
/// `repo_identity::repo_dir_name_for_cwd`). Unset → telemetry no-ops.
static REPO_KEY: OnceLock<String> = OnceLock::new();

/// Cached `BufWriter<File>` opened once on first `append`. Subsequent
/// calls take the lock, `writeln!`, `flush` — one kernel crossing each
/// instead of three (open + write + close).
static WRITER: OnceLock<Option<Mutex<BufWriter<File>>>> = OnceLock::new();

/// Register the per-process repo identity. Called by `ecp-cli` after
/// resolving the canonical repo dir name. Idempotent — second call is a
/// no-op (cannot change identity mid-process). MUST be called before the
/// first `append`, otherwise the writer never opens.
pub fn init_repo_id(repo_key: String) {
    let _ = REPO_KEY.set(repo_key);
}

/// Derive the per-repo telemetry directory.
/// Returns `None` if `init_repo_id` was never called.
fn telemetry_dir() -> Option<PathBuf> {
    let repo_key = REPO_KEY.get()?;
    let base = ecp_core::registry::resolve_home_ecp();
    Some(base.join("telemetry").join(repo_key))
}

/// Lazily open + cache the BufWriter. None means "couldn't open" — every
/// future call short-circuits via the same None (the OnceLock memoises).
fn get_writer() -> Option<&'static Mutex<BufWriter<File>>> {
    WRITER
        .get_or_init(|| {
            let dir = telemetry_dir()?;
            std::fs::create_dir_all(&dir).ok()?;
            let path = dir.join("calls.jsonl");
            OpenOptions::new()
                .create(true)
                .append(true)
                .open(&path)
                .ok()
                .map(BufWriter::new)
                .map(Mutex::new)
        })
        .as_ref()
}

/// Append one jsonl record using the cached writer. All I/O errors are
/// silently discarded — telemetry failure MUST NOT impact MCP dispatch.
pub fn append(record: &CallRecord<'_>) {
    let Some(writer) = get_writer() else { return };
    let Ok(line) = serde_json::to_string(record) else {
        return;
    };
    if let Ok(mut w) = writer.lock() {
        let _ = writeln!(w, "{line}");
        // Flush per-record so crash-safety matches the previous
        // open+write+close behaviour. BufWriter still amortises the
        // user-space buffering (saves us a malloc per call).
        let _ = w.flush();
    }
}

/// Append to an explicit directory, bypassing the cached writer. Used by
/// tests to control the write path without touching real `~/.ecp/`.
pub fn append_to(record: &CallRecord<'_>, dir: &std::path::Path) {
    let _ = append_inner(record, dir);
}

fn append_inner(record: &CallRecord<'_>, dir: &std::path::Path) -> std::io::Result<()> {
    std::fs::create_dir_all(dir)?;
    let path = dir.join("calls.jsonl");
    let line = serde_json::to_string(record).map_err(std::io::Error::other)?;
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    writeln!(file, "{line}")
}
