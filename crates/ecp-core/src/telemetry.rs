//! Shared telemetry record + best-effort jsonl appender.
//!
//! One [`CallRecord`] is appended per invocation — by the CLI (one process
//! per command, file `cli-calls.jsonl`) and by the MCP server (long-lived,
//! file `calls.jsonl`, via its own cached-writer wrapper in ecp-mcp).
//!
//! Schema is **unstable (v1)**. New fields are append-only and optional on
//! read (`#[serde(default)]`) so existing files stay parseable.

use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::Path;

/// One record appended per invocation. CLI and MCP share this exact struct.
#[derive(serde::Serialize)]
pub struct CallRecord<'a> {
    /// RFC3339 UTC timestamp of the call start.
    pub ts: &'a str,
    /// Subcommand (CLI: `"inspect"`) or MCP tool name (`"ecp_inspect"`).
    pub tool: &'a str,
    /// Wall-clock duration in milliseconds.
    pub duration_ms: u64,
    /// `true` on success, `false` on error.
    pub ok: bool,
    /// `"cli"` or `"mcp"`. Distinguishes the two invocation paths.
    pub source: &'a str,
    /// Failure class (e.g. `"no-such-symbol"`); `None` when `ok == true`.
    pub error_kind: Option<&'a str>,
}

/// Append one jsonl line to `dir/filename`. Best-effort: all I/O errors are
/// silently dropped — telemetry MUST NOT affect the caller's result. Single
/// `O_APPEND` write of a sub-PIPE_BUF line is atomic under POSIX, so no lock.
pub fn append_record(dir: &Path, filename: &str, record: &CallRecord<'_>) {
    let Ok(line) = serde_json::to_string(record) else {
        return;
    };
    if std::fs::create_dir_all(dir).is_err() {
        return;
    }
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(dir.join(filename))
    {
        let _ = writeln!(f, "{line}");
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn call_record_serializes_all_fields() {
        let r = CallRecord {
            ts: "2026-05-27T07:00:00Z",
            tool: "inspect",
            duration_ms: 6,
            ok: true,
            source: "cli",
            error_kind: None,
        };
        let line = serde_json::to_string(&r).unwrap();
        assert!(line.contains(r#""source":"cli""#));
        assert!(line.contains(r#""tool":"inspect""#));
        assert!(line.contains(r#""error_kind":null"#));
    }

    #[test]
    fn append_record_writes_one_line() {
        let dir = std::env::temp_dir().join(format!("ecp-tlm-{}", std::process::id()));
        let _ = std::fs::remove_dir_all(&dir);
        let r = CallRecord {
            ts: "2026-05-27T07:00:00Z",
            tool: "find",
            duration_ms: 4,
            ok: false,
            source: "cli",
            error_kind: Some("no-such-symbol"),
        };
        append_record(&dir, "cli-calls.jsonl", &r);
        let body = std::fs::read_to_string(dir.join("cli-calls.jsonl")).unwrap();
        assert_eq!(body.lines().count(), 1);
        assert!(body.contains(r#""error_kind":"no-such-symbol""#));
        let _ = std::fs::remove_dir_all(&dir);
    }
}
