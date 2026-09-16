//! Terminal UI — Flutter-style CLI output (colors, banners, steps).

use std::io::{self, Write};

pub const VERSION: &str = env!("CARGO_PKG_VERSION");

pub fn welcome() {
    banner();
    println!();
    dim("  Get started:");
    println!("    {}  Create a new app", cmd("flash create my_app"));
    println!("    {}  Check your setup", cmd("flash doctor"));
    println!("    {}  View documentation", cmd("flash docs"));
    println!();
    dim("  Install SDK:");
    println!("    curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash");
    println!();
}

pub fn project_welcome(name: &str, entry: &str) {
    banner();
    println!();
    success(&format!("Project: {}", name));
    dim(&format!("  Entry: {}", entry));
    println!();
    dim("  Available commands:");
    println!("    {}  Hot reload (recommended)", cmd("flash run"));
    println!("    {}  Native hot reload", cmd("flash run -d native"));
    println!("    {}  One-shot simulate", cmd("flash run --once"));
    println!("    {}  List targets", cmd("flash devices"));
    println!("    {}  Fetch Rust deps", cmd("flash pub get"));
    println!();
}

pub fn banner() {
    println!(
        "{}⚡ {}Flash{} {}{}",
        BOLD,
        CYAN,
        RESET,
        VERSION,
        RESET
    );
    dim("  Compiled UI for iOS, Android & Web — native widgets, zero JS bridge");
}

pub fn create_header(name: &str) {
    banner();
    println!();
    step(&format!("Creating project {}...", name));
}

pub fn create_done(name: &str) {
    println!();
    success("All done!");
    println!();
    println!("  In order to run your application, type:");
    println!();
    println!("    {}", cmd(&format!("cd {}", name)));
    println!("    {}", cmd("flash run"));
    println!();
    dim("  Hot reload is on by default — save any .ui file to see changes.");
    println!();
}

pub fn launch_line(entry: &str, device: &str) {
    println!();
    step(&format!(
        "Launching {} on {} in debug mode...",
        entry, device
    ));
}

pub fn dev_ready(entry: &str, device: &str) {
    println!();
    success("Application running.");
    dim(&format!("  Screen: {}", entry));
    dim(&format!("  Device: {}", device));
    println!();
    dim("  Hot reload: save your .ui file — changes apply in <1s");
    dim("  Press Ctrl+C to quit.");
    println!();
}

pub fn step(msg: &str) {
    println!("{}  {}{}", CYAN, msg, RESET);
}

pub fn success(msg: &str) {
    println!("{}  ✓ {}{}", GREEN, msg, RESET);
}

pub fn warn(msg: &str) {
    println!("{}  ! {}{}", YELLOW, msg, RESET);
}

pub fn error(msg: &str) {
    eprintln!("{}  ✗ {}{}", RED, msg, RESET);
}

pub fn dim(msg: &str) {
    println!("{}{}{}", DIM, msg, RESET);
}

pub fn cmd(s: &str) -> String {
    format!("{}{}{}", BOLD, s, RESET)
}

pub fn flush() {
    io::stdout().flush().ok();
}

const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";
const CYAN: &str = "\x1b[36m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const RED: &str = "\x1b[31m";
const RESET: &str = "\x1b[0m";
