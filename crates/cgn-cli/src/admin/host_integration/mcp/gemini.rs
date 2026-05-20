//! Gemini CLI MCP integration.

use crate::admin::status::HostStatus;
use dialoguer::theme::ColorfulTheme;
use std::env;
use std::process::Command;

const SERVER_NAME: &str = "cgn";

pub fn install(_theme: &ColorfulTheme) {
    let exe = match env::current_exe() {
        Ok(p) => p.to_string_lossy().into_owned(),
        Err(e) => {
            eprintln!("Error: Could not determine current executable path: {}", e);
            return;
        }
    };

    println!("Registering cgn MCP server in Gemini CLI...");
    let output = Command::new("gemini")
        .args(["mcp", "add", SERVER_NAME, &exe, "admin", "mcp", "serve"])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            println!("✓ Gemini CLI MCP server 'cgn' added successfully.")
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            eprintln!("Error adding MCP server: {}", err);
        }
        Err(e) => eprintln!("Failed to spawn gemini: {}", e),
    }
}

pub fn uninstall(_theme: &ColorfulTheme) {
    let output = Command::new("gemini")
        .args(["mcp", "remove", SERVER_NAME])
        .output();

    match output {
        Ok(o) if o.status.success() => println!("✓ Gemini CLI MCP server 'cgn' removed."),
        Ok(_) => println!("Gemini CLI MCP server 'cgn' was not found or already removed."),
        Err(e) => eprintln!("Failed to spawn gemini: {}", e),
    }
}

pub fn status() -> HostStatus {
    let output = Command::new("gemini").args(["mcp", "list"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let list = String::from_utf8_lossy(&o.stdout);
            if list.contains(SERVER_NAME) {
                HostStatus::Installed {
                    detail: "managed via gemini mcp".into(),
                }
            } else {
                HostStatus::Missing
            }
        }
        _ => HostStatus::Missing,
    }
}
