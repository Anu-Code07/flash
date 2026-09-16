//! `flash upgrade` — update SDK and CLI to the latest version.

use std::env;
use std::path::Path;
use std::process::Command;

use crate::ui;

pub fn run_upgrade() {
    ui::banner();
    println!();

    let sdk = env::var("FLASH_SDK").unwrap_or_else(|_| {
        let home = env::var("HOME").unwrap_or_default();
        format!("{}/.flash/sdk", home)
    });

    let sdk_path = Path::new(&sdk);
    if !sdk_path.join(".git").exists() {
        ui::error(&format!("Flash SDK not found at {}", sdk));
        ui::dim("  curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash");
        std::process::exit(1);
    }

    ui::step("Pulling latest Flash SDK...");
    let pull = Command::new("git")
        .args(["-C", &sdk, "pull", "origin", "main", "--ff-only"])
        .status();
    match pull {
        Ok(s) if s.success() => ui::success("SDK updated"),
        Ok(_) => {
            ui::warn("git pull failed — check $FLASH_SDK for local changes");
        }
        Err(e) => {
            ui::error(&format!("git not available: {}", e));
            std::process::exit(1);
        }
    }

    ui::step("Rebuilding flash CLI...");
    let install = Command::new("cargo")
        .args(["install", "--path", &format!("{}/cli", sdk), "--force"])
        .status();
    match install {
        Ok(s) if s.success() => {
            ui::success("CLI upgraded");
            println!();
            ui::dim("  Run flash doctor to verify");
            println!();
        }
        Ok(_) => {
            ui::error("cargo install failed");
            std::process::exit(1);
        }
        Err(e) => {
            ui::error(&format!("cargo not available: {}", e));
            std::process::exit(1);
        }
    }
}
