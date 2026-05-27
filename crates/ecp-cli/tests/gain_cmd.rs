use std::process::Command;

fn ecp_bin() -> &'static str {
    env!("CARGO_BIN_EXE_ecp")
}

#[test]
fn invocation_appends_one_cli_telemetry_line() {
    let tmp = std::env::temp_dir().join(format!("ecp-gain-it-{}", std::process::id()));
    let _ = std::fs::remove_dir_all(&tmp);
    std::fs::create_dir_all(&tmp).unwrap();
    let out = Command::new(ecp_bin())
        .args(["find", "definitely_no_such_symbol_xyz"])
        .current_dir(&tmp)
        .env("HOME", &tmp)
        .env_remove("ECP_NO_TELEMETRY")
        .output()
        .unwrap();
    let _ = out; // command may fail (no graph) — we only assert telemetry wrote
    let tel_root = tmp.join(".ecp/telemetry");
    let mut found = false;
    if let Ok(entries) = std::fs::read_dir(&tel_root) {
        for e in entries.flatten() {
            let f = e.path().join("cli-calls.jsonl");
            if f.exists() {
                let body = std::fs::read_to_string(&f).unwrap();
                assert!(body.lines().count() >= 1, "expected >=1 telemetry line");
                assert!(body.contains(r#""source":"cli""#));
                found = true;
            }
        }
    }
    assert!(found, "no cli-calls.jsonl written under {tel_root:?}");
    let _ = std::fs::remove_dir_all(&tmp);
}
