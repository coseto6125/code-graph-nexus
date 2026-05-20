//! Gemini-specific admin workflows.

use crate::admin::host_integration::mcp::gemini as mcp_gemini;
use crate::admin::host_integration::native::gemini as native_gemini;
use crate::admin::menu::{self, select};
use crate::admin::status::HostStatus;
use cgn_core::CgnError;
use dialoguer::theme::ColorfulTheme;

const MENU: &[menu::Item<'_>] = &[
    (
        "Native Install",
        "register cgn as a native skill in Gemini CLI",
    ),
    ("MCP Install", "register cgn as an MCP server in Gemini CLI"),
    ("Status", "check Gemini CLI integration status"),
    ("Uninstall", "remove all cgn integrations from Gemini CLI"),
    ("← Back", ""),
];

pub fn run(theme: &ColorfulTheme) -> Result<(), CgnError> {
    loop {
        let choice = select(theme, "Gemini Configuration", MENU)?;
        match choice {
            Some(0) => native_gemini::install(theme),
            Some(1) => mcp_gemini::install(theme),
            Some(2) => print_status(),
            Some(3) => uninstall_all(theme),
            Some(4) | None => return Ok(()),
            _ => unreachable!(),
        }
    }
}

fn print_status() {
    println!("Gemini CLI Integration Status:");
    native_gemini::status().print("Native Skill");
    mcp_gemini::status().print("MCP Server");
}

fn uninstall_all(theme: &ColorfulTheme) {
    native_gemini::uninstall(theme);
    mcp_gemini::uninstall(theme);
}
