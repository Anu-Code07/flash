//! Flash project — reads `flash.toml` for app name, entry screen, and paths.

use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::{self, Command};

use crate::ui;

#[derive(Clone, Debug)]
pub struct FlashProject {
    pub name: String,
    pub entry: PathBuf,
    pub root: PathBuf,
}

/// Load project from cwd `flash.toml`.
pub fn load() -> Option<FlashProject> {
    let root = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
    let toml = root.join("flash.toml");
    if !toml.is_file() {
        return None;
    }
    let text = fs::read_to_string(&toml).ok()?;
    Some(FlashProject {
        name: parse_toml_string(&text, "name").unwrap_or_else(|| "app".to_string()),
        entry: PathBuf::from(
            parse_toml_string(&text, "entry").unwrap_or_else(|| "ui/screens/home.ui".to_string()),
        ),
        root,
    })
}

/// Resolve `.ui` path: explicit arg, or `flash.toml` entry.
pub fn resolve_ui(path: Option<&Path>) -> PathBuf {
    if let Some(p) = path {
        return p.to_path_buf();
    }
    if let Some(proj) = load() {
        return proj.entry;
    }
    ui::error("Not in a Flash project and no .ui file given.");
    println!();
    dim_hint();
    process::exit(1);
}

pub fn require_project() -> FlashProject {
    load().unwrap_or_else(|| {
        ui::error("No flash.toml found. Run from a project directory or use flash create.");
        println!();
        dim_hint();
        process::exit(1);
    })
}

/// `flash pub get` — fetch workspace dependencies.
pub fn pub_get() {
    let proj = require_project();
    ui::step("Resolving dependencies...");
    let cargo_toml = proj.root.join("Cargo.toml");
    if !cargo_toml.is_file() {
        ui::warn("No Cargo.toml — skipping Rust deps");
        return;
    }
    let status = Command::new("cargo")
        .arg("fetch")
        .current_dir(&proj.root)
        .status();
    match status {
        Ok(s) if s.success() => ui::success("Got dependencies!"),
        Ok(_) => {
            ui::error("cargo fetch failed");
            process::exit(1);
        }
        Err(e) => {
            ui::error(&format!("cargo not available: {}", e));
            process::exit(1);
        }
    }
}

fn dim_hint() {
    ui::dim("  flash create my_app && cd my_app && flash run");
}

fn parse_toml_string(text: &str, key: &str) -> Option<String> {
    let needle = format!("{} =", key);
    let needle_q = format!("{}=", key);
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with(&needle) || line.starts_with(&needle_q) {
            if let Some((_, val)) = line.split_once('=') {
                return Some(val.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}

/// Back-compat for doctor module.
pub fn read_flash_entry(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .and_then(|text| parse_toml_string(&text, "entry"))
}
