//! `flash doctor` — verify toolchain like `flutter doctor`.

use std::env;
use std::path::Path;
use std::process::Command;

struct Check {
    name: &'static str,
    ok: bool,
    detail: String,
    fix: Option<&'static str>,
}

pub fn run_doctor() {
    let checks = collect_checks();
    let mut failures = 0;

    println!("Flash Doctor — checking your environment\n");

    for c in &checks {
        let icon = if c.ok { "✓" } else { "✗" };
        println!("  [{}] {} — {}", icon, c.name, c.detail);
        if !c.ok {
            failures += 1;
            if let Some(fix) = c.fix {
                println!("      → {}", fix);
            }
        }
    }

    println!();
    if failures == 0 {
        println!("All checks passed! You're ready to build Flash apps.");
        println!("\n  flash create my_app");
        println!("  cd my_app && flash dev");
    } else {
        println!(
            "{} issue(s) found. Fix the items above, then run `flash doctor` again.",
            failures
        );
        std::process::exit(1);
    }
}

fn collect_checks() -> Vec<Check> {
    vec![
        check_flash_cli(),
        check_rustc(),
        check_cargo_path(),
        check_flash_sdk(),
        check_project(),
        check_ios(),
        check_android(),
    ]
}

fn check_flash_cli() -> Check {
    let ok = Command::new("flash")
        .arg("--help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    Check {
        name: "Flash CLI",
        ok,
        detail: if ok {
            "installed".into()
        } else {
            "not found on PATH".into()
        },
        fix: Some("curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash"),
    }
}

fn check_rustc() -> Check {
    match Command::new("rustc").arg("--version").output() {
        Ok(o) if o.status.success() => {
            let ver = String::from_utf8_lossy(&o.stdout).trim().to_string();
            let ok = ver.contains("1.7") || ver.contains("1.8") || ver.contains("1.9");
            Check {
                name: "Rust",
                ok,
                detail: ver,
                fix: Some("curl https://sh.rustup.rs | sh"),
            }
        }
        _ => Check {
            name: "Rust",
            ok: false,
            detail: "rustc not found".into(),
            fix: Some("curl https://sh.rustup.rs | sh"),
        },
    }
}

fn check_cargo_path() -> Check {
    let flash_works = Command::new("flash")
        .arg("help")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false);
    let home = env::var("HOME").unwrap_or_default();
    let cargo_bin = format!("{}/.cargo/bin", home);
    let path = env::var("PATH").unwrap_or_default();
    let on_path = path.contains(".cargo/bin") || path.contains(&cargo_bin);
    let ok = flash_works || on_path;
    Check {
        name: "PATH",
        ok,
        detail: if flash_works {
            "flash command available".into()
        } else if on_path {
            "~/.cargo/bin on PATH".into()
        } else {
            "~/.cargo/bin missing from PATH".into()
        },
        fix: Some("echo 'export PATH=\"$HOME/.cargo/bin:$PATH\"' >> ~/.zshrc && source ~/.zshrc"),
    }
}

fn check_flash_sdk() -> Check {
    let sdk = env::var("FLASH_SDK").unwrap_or_else(|_| {
        let home = env::var("HOME").unwrap_or_default();
        format!("{}/.flash/sdk", home)
    });
    let in_repo = Path::new("cli/Cargo.toml").is_file();
    let sdk_ok = Path::new(&sdk).join("cli/Cargo.toml").is_file();
    let ok = sdk_ok || in_repo;
    Check {
        name: "Flash SDK",
        ok,
        detail: if sdk_ok {
            format!("FLASH_SDK={}", sdk)
        } else if in_repo {
            "using Flash git repo (dev)".into()
        } else {
            format!("not found at {} (needed for device builds)", sdk)
        },
        fix: Some("curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash"),
    }
}

fn check_project() -> Check {
    let toml = Path::new("flash.toml");
    if !toml.is_file() {
        return Check {
            name: "Project",
            ok: true,
            detail: "not inside a Flash app (ok)".into(),
            fix: None,
        };
    }
    let entry = read_flash_entry(toml);
    let entry_path = entry.as_deref().unwrap_or("ui/screens/home.ui");
    let ok = Path::new(entry_path).is_file();
    Check {
        name: "Project",
        ok,
        detail: if ok {
            format!("flash.toml → {}", entry_path)
        } else {
            format!("entry missing: {}", entry_path)
        },
        fix: Some("flash create my_app && cd my_app"),
    }
}

fn check_ios() -> Check {
    if !cfg!(target_os = "macos") {
        return Check {
            name: "iOS (Xcode)",
            ok: true,
            detail: "skipped (not macOS)".into(),
            fix: None,
        };
    }
    let xcode = Command::new("xcode-select").arg("-p").output();
    let ok = xcode.map(|o| o.status.success()).unwrap_or(false);
    Check {
        name: "iOS (Xcode)",
        ok,
        detail: if ok {
            "Xcode command-line tools".into()
        } else {
            "Xcode not installed".into()
        },
        fix: Some("xcode-select --install"),
    }
}

fn check_android() -> Check {
    let ndk = env::var("ANDROID_NDK_HOME")
        .or_else(|_| env::var("NDK_HOME"))
        .ok();
    let sdk = env::var("ANDROID_HOME")
        .or_else(|_| env::var("ANDROID_SDK_ROOT"))
        .ok();
    let configured = ndk.is_some() || sdk.is_some();
    Check {
        name: "Android (optional)",
        ok: true,
        detail: if configured {
            "ANDROID_HOME or NDK configured".into()
        } else {
            "not configured — ok for .ui dev, needed for Android builds".into()
        },
        fix: None,
    }
}

/// Minimal `entry = "..."` parse from flash.toml (no extra deps).
pub fn read_flash_entry(path: &Path) -> Option<String> {
    let text = std::fs::read_to_string(path).ok()?;
    for line in text.lines() {
        let line = line.trim();
        if line.starts_with("entry") {
            if let Some((_, val)) = line.split_once('=') {
                return Some(val.trim().trim_matches('"').to_string());
            }
        }
    }
    None
}
