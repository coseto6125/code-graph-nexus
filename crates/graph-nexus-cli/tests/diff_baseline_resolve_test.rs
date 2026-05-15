//! Verify `gnx diff` CLI surface: required args, section enum, baseline rejection.

use std::process::Command;

fn gnx_bin() -> &'static str {
    env!("CARGO_BIN_EXE_gnx")
}

#[test]
fn diff_requires_section_and_baseline() {
    let output = Command::new(gnx_bin())
        .args(["diff"])
        .output()
        .expect("run gnx diff");
    assert!(!output.status.success(), "diff without args must reject");
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(
        stderr.contains("--section") || stderr.contains("section"),
        "missing-section hint expected, got stderr: {stderr}"
    );
    assert!(
        stderr.contains("--baseline") || stderr.contains("baseline"),
        "missing-baseline hint expected, got stderr: {stderr}"
    );
}

#[test]
fn diff_help_lists_section_choices() {
    let output = Command::new(gnx_bin())
        .args(["diff", "--help"])
        .output()
        .expect("run gnx diff --help");
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    for word in ["bindings", "routes", "contracts", "all"] {
        assert!(
            stdout.contains(word),
            "expected `{word}` in --help possible values, got: {stdout}"
        );
    }
}
