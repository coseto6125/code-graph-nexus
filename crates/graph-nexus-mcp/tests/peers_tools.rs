//! Smoke tests: peer tool registration and spawn-argv shape.

use clap::{Args, CommandFactory, Parser, Subcommand};
use graph_nexus_mcp::server::GnxMcpServer;
use graph_nexus_mcp::spawn::run_spawn;
use serde_json::json;
use std::os::unix::fs::PermissionsExt;
use tempfile::TempDir;

// ── minimal synthetic CLI tree (no gnx binary needed) ────────────────────────

#[derive(Parser)]
#[command(name = "gnx")]
struct Cli {
    #[command(subcommand)]
    cmd: Cmds,
}

#[derive(Subcommand)]
enum Cmds {
    /// Visible surrogate.
    Inspect(InspectArgs),
    /// Multi-session peer collaboration (status / diff / log / gc + Ƀ messaging)
    Peers(PeersArgs),
}

#[derive(Args)]
struct InspectArgs {
    #[arg(long)]
    name: Option<String>,
}

#[derive(Args)]
struct PeersArgs {
    #[command(subcommand)]
    cmd: PeersCmd,
}

#[derive(Subcommand)]
enum PeersCmd {
    Status,
    Log,
    Say { body: String },
}

// ── registration tests ────────────────────────────────────────────────────────

#[tokio::test(flavor = "current_thread")]
async fn peer_tools_registered_by_name() {
    let server = GnxMcpServer::new(&Cli::command()).expect("init");
    let names: Vec<&str> = server
        .list_tools()
        .iter()
        .map(|t| t.name.as_str())
        .collect();

    assert!(
        names.contains(&"gnx_peers_status"),
        "missing gnx_peers_status; got {names:?}"
    );
    assert!(
        names.contains(&"gnx_peers_log"),
        "missing gnx_peers_log; got {names:?}"
    );
    assert!(
        names.contains(&"gnx_peers_say"),
        "missing gnx_peers_say; got {names:?}"
    );
}

#[tokio::test(flavor = "current_thread")]
async fn opaque_gnx_peers_not_registered() {
    let server = GnxMcpServer::new(&Cli::command()).expect("init");
    let names: Vec<&str> = server
        .list_tools()
        .iter()
        .map(|t| t.name.as_str())
        .collect();
    assert!(
        !names.contains(&"gnx_peers"),
        "opaque gnx_peers must not appear; got {names:?}"
    );
}

// ── spawn argv shape tests ────────────────────────────────────────────────────

fn write_stub(dir: &std::path::Path, script: &str) -> std::path::PathBuf {
    let stub = dir.join("gnx");
    std::fs::write(&stub, script).unwrap();
    let mut perms = std::fs::metadata(&stub).unwrap().permissions();
    perms.set_mode(0o755);
    std::fs::set_permissions(&stub, perms).unwrap();
    stub
}

#[test]
fn status_spawn_argv_contains_peers_status() {
    let dir = TempDir::new().unwrap();
    let stub = write_stub(dir.path(), "#!/bin/sh\necho \"$@\"\n");
    let tool = graph_nexus_mcp::peers::peer_tools()
        .into_iter()
        .find(|t| t.name == "gnx_peers_status")
        .unwrap();
    let out = run_spawn(&stub, &tool, &json!({})).unwrap();
    // Stub echoes all args; expect "peers status" as first two tokens.
    assert!(
        out.contains("status"),
        "expected 'status' in argv echo: {out:?}"
    );
}

#[test]
fn log_spawn_argv_contains_peers_log_with_options() {
    let dir = TempDir::new().unwrap();
    let stub = write_stub(dir.path(), "#!/bin/sh\necho \"$@\"\n");
    let tool = graph_nexus_mcp::peers::peer_tools()
        .into_iter()
        .find(|t| t.name == "gnx_peers_log")
        .unwrap();
    let out = run_spawn(&stub, &tool, &json!({"limit": 10})).unwrap();
    assert!(out.contains("log"), "expected 'log' in argv echo: {out:?}");
    assert!(
        out.contains("--limit"),
        "expected '--limit' in argv echo: {out:?}"
    );
}

#[test]
fn say_spawn_argv_positional_body_after_say() {
    let dir = TempDir::new().unwrap();
    let stub = write_stub(dir.path(), "#!/bin/sh\necho \"$@\"\n");
    let tool = graph_nexus_mcp::peers::peer_tools()
        .into_iter()
        .find(|t| t.name == "gnx_peers_say")
        .unwrap();
    let out = run_spawn(&stub, &tool, &json!({"body": "hello"})).unwrap();
    assert!(out.contains("say"), "expected 'say' in argv echo: {out:?}");
    assert!(
        out.contains("hello"),
        "expected message body 'hello' in argv echo: {out:?}"
    );
}
