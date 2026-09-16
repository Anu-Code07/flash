//! `flash devices` — list run targets (like `flutter devices`).

use std::env;
use std::process::Command;

use crate::ui;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Device {
    Simulator,
    Native,
    Ios,
    Android,
}

impl Device {
    pub fn id(self) -> &'static str {
        match self {
            Device::Simulator => "simulator",
            Device::Native => "native",
            Device::Ios => "ios",
            Device::Android => "android",
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Device::Simulator => "Simulator (in-process)",
            Device::Native => "Native (command buffer)",
            Device::Ios => "iOS (UIKit)",
            Device::Android => "Android (Views)",
        }
    }

    pub fn parse(s: &str) -> Option<Self> {
        match s {
            "simulator" | "sim" | "desktop" => Some(Device::Simulator),
            "native" => Some(Device::Native),
            "ios" => Some(Device::Ios),
            "android" => Some(Device::Android),
            _ => None,
        }
    }

    pub fn default() -> Self {
        Device::Simulator
    }
}

pub fn list_devices() {
    ui::banner();
    println!();
    ui::step("Connected devices:");
    println!();

    let default = Device::default();
    for d in [
        Device::Simulator,
        Device::Native,
        Device::Ios,
        Device::Android,
    ] {
        let avail = device_available(d);
        let mark = if d == default { " (default)" } else { "" };
        let status = if avail {
            format!("{}●{}", "\x1b[32m", "\x1b[0m")
        } else {
            format!("{}○{}", "\x1b[2m", "\x1b[0m")
        };
        println!(
            "  {} {:12}  {}{}",
            status,
            d.id(),
            d.label(),
            mark
        );
        if !avail {
            ui::dim(&format!("      {}", unavailable_reason(d)));
        }
    }
    println!();
    ui::dim("  Usage: flash run -d <device>   flash dev -d native");
    println!();
}

fn device_available(d: Device) -> bool {
    match d {
        Device::Simulator | Device::Native => true,
        Device::Ios => cfg!(target_os = "macos") && Command::new("xcode-select").arg("-p").status().map(|s| s.success()).unwrap_or(false),
        Device::Android => env::var("ANDROID_HOME").is_ok() || env::var("ANDROID_SDK_ROOT").is_ok(),
    }
}

fn unavailable_reason(d: Device) -> String {
    match d {
        Device::Ios => "Requires macOS + Xcode (xcode-select --install)".into(),
        Device::Android => "Set ANDROID_HOME (install Android Studio)".into(),
        _ => String::new(),
    }
}
