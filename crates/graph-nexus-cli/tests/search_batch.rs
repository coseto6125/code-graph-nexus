//! `gnx search --batch` reads patterns from stdin (one per line, `#`
//! comments and empty lines skipped) and emits a per-query block
//! prefixed by `=== pattern: <pattern> ===`. The point is to amortise
//! the BGE-M3 cold start across N queries inside a single process.

use std::io::Write;
use std::process::{Command, Stdio};

fn gnx_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gnx")
}

fn run_batch_with_stdin(stdin_payload: &str, extra_args: &[&str]) -> std::process::Output {
    let mut args = vec!["search", "--batch", "--mode", "bm25"];
    args.extend_from_slice(extra_args);
    let mut child = Command::new(gnx_bin())
        .args(args)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .spawn()
        .unwrap();
    child
        .stdin
        .as_mut()
        .unwrap()
        .write_all(stdin_payload.as_bytes())
        .unwrap();
    child.wait_with_output().unwrap()
}

#[test]
fn batch_emits_per_query_divider_lines() {
    let payload = "rrf_merge\ncosine_top_k_indices\n";
    let out = run_batch_with_stdin(payload, &[]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    // Each query's block must start with the divider.
    assert!(
        stdout.contains("=== pattern: rrf_merge ==="),
        "missing divider for rrf_merge in:\n{stdout}"
    );
    assert!(
        stdout.contains("=== pattern: cosine_top_k_indices ==="),
        "missing divider for cosine_top_k_indices in:\n{stdout}"
    );
}

#[test]
fn batch_skips_blank_and_commented_lines() {
    // 5 lines on stdin but only 1 is a real query.
    let payload = "\n# this is a comment\n   \nrrf_merge\n# trailing comment\n";
    let out = run_batch_with_stdin(payload, &[]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let divider_count = stdout.matches("=== pattern: ").count();
    assert_eq!(
        divider_count, 1,
        "expected exactly 1 divider (only 'rrf_merge' is a real query), got {divider_count} in:\n{stdout}"
    );
    assert!(stdout.contains("=== pattern: rrf_merge ==="));
}

#[test]
fn batch_with_empty_stdin_emits_no_query_dividers() {
    let payload = "\n# only comments\n\n";
    let out = run_batch_with_stdin(payload, &[]);
    let stdout = String::from_utf8_lossy(&out.stdout);
    let stderr = String::from_utf8_lossy(&out.stderr);
    assert!(
        !stdout.contains("=== pattern: "),
        "expected no query dividers, got:\n{stdout}"
    );
    // The non-empty-input contract: emit a one-line stderr hint.
    assert!(
        stderr.contains("batch: no patterns on stdin"),
        "expected stderr hint, got:\n{stderr}"
    );
}
