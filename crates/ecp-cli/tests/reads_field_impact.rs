//! End-to-end: `ecp impact <field>` reaches the field's readers via the
//! ReadsField edge. This is the LLM-utility payoff — before this edge the
//! query returned empty (indistinguishable from "no impact"), so a refactor
//! changing a field silently missed its readers.

use serde_json::Value;
use std::path::Path;
use std::process::Command;

fn ecp_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ecp")
}

const SRC: &str = r#"
pub struct Config {
    pub timeout: u32,
}

pub fn read_timeout(c: &Config) -> u32 {
    c.timeout
}
"#;

fn init_and_index(repo: &Path) {
    Command::new("git")
        .args(["init", "-q", "-b", "main"])
        .current_dir(repo)
        .output()
        .unwrap();
    std::fs::create_dir_all(repo.join("src")).unwrap();
    std::fs::write(repo.join("src/lib.rs"), SRC).unwrap();
    Command::new("git")
        .args([
            "-c",
            "user.email=t@t",
            "-c",
            "user.name=t",
            "commit",
            "--allow-empty",
            "-q",
            "-m",
            "init",
        ])
        .current_dir(repo)
        .output()
        .unwrap();
    let out = Command::new(ecp_bin())
        .args(["admin", "index", "--repo", "."])
        .current_dir(repo)
        .env("HOME", repo)
        .output()
        .expect("admin index spawn");
    assert!(
        out.status.success(),
        "admin index failed: {}",
        String::from_utf8_lossy(&out.stderr)
    );
}

#[test]
fn impact_on_field_reaches_reader() {
    let tmp = tempfile::tempdir().unwrap();
    let repo = tmp.path();
    init_and_index(repo);

    let out = Command::new(ecp_bin())
        .args(["impact", "timeout", "--repo", ".", "--format", "json"])
        .current_dir(repo)
        .env("HOME", repo)
        .output()
        .expect("impact spawn");
    let stdout = String::from_utf8_lossy(&out.stdout);
    let json_start = stdout
        .find('{')
        .unwrap_or_else(|| panic!("impact returned no JSON\nstdout={stdout}"));
    let json: Value = serde_json::from_str(&stdout[json_start..]).unwrap();

    let reaches_reader = json["impact"]
        .as_array()
        .unwrap_or(&vec![])
        .iter()
        .any(|e| e["name"].as_str() == Some("read_timeout"));

    assert!(
        reaches_reader,
        "ecp impact timeout must reach read_timeout via ReadsField.\nstdout={stdout}"
    );
}
