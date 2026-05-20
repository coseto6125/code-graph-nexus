//! Gemini CLI native integration (Skill-based).

use crate::admin::status::HostStatus;
use dialoguer::theme::ColorfulTheme;
use std::env;
use std::process::Command;

const SKILL_NAME: &str = "cgn";

pub fn install(_theme: &ColorfulTheme) {
    let skill_path = match find_skill_path() {
        Some(p) => p,
        None => {
            eprintln!(
                "Error: Could not find cgn skill source (expected docs/skills/cgn in repo root)."
            );
            return;
        }
    };

    println!("Installing Gemini CLI native skill from {}...", skill_path);
    let output = Command::new("gemini")
        .args(["skills", "link", &skill_path])
        .output();

    match output {
        Ok(o) if o.status.success() => {
            println!("✓ Gemini CLI native skill 'cgn' linked successfully.")
        }
        Ok(o) => {
            let err = String::from_utf8_lossy(&o.stderr);
            eprintln!("Error linking skill: {}", err);
        }
        Err(e) => eprintln!("Failed to spawn gemini: {}", e),
    }
}

pub fn uninstall(_theme: &ColorfulTheme) {
    let output = Command::new("gemini")
        .args(["skills", "uninstall", SKILL_NAME])
        .output();

    match output {
        Ok(o) if o.status.success() => println!("✓ Gemini CLI native skill 'cgn' uninstalled."),
        Ok(_) => println!("Gemini CLI native skill 'cgn' was not installed or already removed."),
        Err(e) => eprintln!("Failed to spawn gemini: {}", e),
    }
}

pub fn status() -> HostStatus {
    let output = Command::new("gemini").args(["skills", "list"]).output();

    match output {
        Ok(o) if o.status.success() => {
            let list = String::from_utf8_lossy(&o.stdout);
            if list.contains(SKILL_NAME) {
                HostStatus::Installed {
                    detail: "linked via gemini skills link".into(),
                }
            } else {
                HostStatus::Missing
            }
        }
        _ => HostStatus::Missing,
    }
}

fn find_skill_path() -> Option<String> {
    let current = env::current_dir().ok()?;
    let mut dir = current.clone();
    loop {
        let path = dir.join("docs/skills/cgn");
        if path.exists() {
            return Some(path.to_string_lossy().into_owned());
        }
        if !dir.pop() {
            break;
        }
    }
    None
}
