//! `flash dev` — watch `.ui` files, hot reload on save.

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime};

use flash_driver::compile;
use flash_runtime::{DevSession, HotReloadKind};

const POLL_MS: u64 = 300;

pub fn run_dev(path: &Path) {
    let initial = fs::read_to_string(path).expect("failed to read .ui file");
    let result = compile(&initial).expect("initial compile failed");
    let screen = result
        .ir
        .screens
        .first()
        .cloned()
        .expect("no screen in .ui file");

    let mut session = DevSession::mount(screen);
    let mut last_mtime = file_mtime(path);
    let mut generation = 1u32;

    println!("⚡ Flash dev — hot reload enabled");
    println!("   Watching: {}", path.display());
    println!("   Edit the .ui file and save — changes apply in <1s");
    println!("   Press Ctrl+C to stop\n");
    print_session(&session, generation, "Mounted");

    loop {
        thread::sleep(Duration::from_millis(POLL_MS));
        let mtime = file_mtime(path);
        if mtime <= last_mtime {
            continue;
        }
        last_mtime = mtime;

        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("✗ Read error: {}", e);
                continue;
            }
        };

        let compiled = match compile(&source) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("✗ Compile error (state preserved):\n{}", e);
                continue;
            }
        };

        let new_screen = match compiled.ir.screens.first() {
            Some(s) => s.clone(),
            None => {
                eprintln!("✗ No screen found in .ui file");
                continue;
            }
        };

        let reload = session.apply(new_screen);
        generation += 1;

        let label = match reload.kind {
            HotReloadKind::HotReload => {
                format!(
                    "Hot reload — {} prop(s) patched, state preserved",
                    reload.props_patched
                )
            }
            HotReloadKind::HotRestart => {
                format!(
                    "Hot restart — tree remounted, {} slot(s) preserved",
                    session.slot_values().len()
                )
            }
        };
        print_session(&session, generation, &label);
    }
}

fn print_session(session: &DevSession, generation: u32, event: &str) {
    println!("── gen {} ── {}", generation, event);
    for line in session.renderer.describe() {
        println!("{}", line);
    }
    if !session.slot_values().is_empty() {
        print!("   slots:");
        for (i, v) in session.slot_values().iter().enumerate() {
            print!(" slot{}={}", i, v);
        }
        println!();
    }
    io::stdout().flush().ok();
    println!();
}

fn file_mtime(path: &Path) -> SystemTime {
    fs::metadata(path)
        .and_then(|m| m.modified())
        .unwrap_or(SystemTime::UNIX_EPOCH)
}
