//! `flash dev` — watch `.ui` files, hot reload on save.

use std::fs;
use std::io::{self, Write};
use std::path::Path;
use std::thread;
use std::time::{Duration, SystemTime};

use flash_driver::compile;
use flash_runtime::{DevSession, HotReloadKind, NativeDevSession};

const POLL_MS: u64 = 300;

pub fn run_dev(path: &Path, native: bool) {
    let initial = fs::read_to_string(path).expect("failed to read .ui file");
    let result = compile(&initial).expect("initial compile failed");
    let screen = result
        .ir
        .screens
        .first()
        .cloned()
        .expect("no screen in .ui file");

    if native {
        run_native_dev(path, screen);
    } else {
        run_mock_dev(path, screen);
    }
}

fn run_mock_dev(path: &Path, screen: flash_ir::ScreenIr) {
    let mut session = DevSession::mount(screen);
    let mut last_mtime = file_mtime(path);
    let mut generation = 1u32;

    println!("⚡ Flash dev — hot reload enabled");
    println!("   Watching: {}", path.display());
    println!("   Edit the .ui file and save — changes apply in <1s");
    println!("   Press Ctrl+C to stop\n");
    print_mock_session(&session, generation, "Mounted");

    watch_loop(path, &mut last_mtime, || {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("✗ Read error: {}", e);
                return;
            }
        };
        let compiled = match compile(&source) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("✗ Compile error (state preserved):\n{}", e);
                return;
            }
        };
        let new_screen = match compiled.ir.screens.first() {
            Some(s) => s.clone(),
            None => {
                eprintln!("✗ No screen found in .ui file");
                return;
            }
        };
        let reload = session.apply(new_screen);
        generation += 1;
        print_mock_session(&session, generation, &reload_label(&reload, &session));
    });
}

fn run_native_dev(path: &Path, screen: flash_ir::ScreenIr) {
    let mut session = NativeDevSession::mount(screen);
    let mut last_mtime = file_mtime(path);
    let mut generation = 1u32;

    println!("⚡ Flash dev — native hot reload enabled");
    println!("   Watching: {}", path.display());
    println!("   Patches flow through flash_host_apply_ops (in-process host)");
    println!("   Press Ctrl+C to stop\n");
    print_native_session(&session, generation, "Mounted");

    watch_loop(path, &mut last_mtime, || {
        let source = match fs::read_to_string(path) {
            Ok(s) => s,
            Err(e) => {
                eprintln!("✗ Read error: {}", e);
                return;
            }
        };
        let compiled = match compile(&source) {
            Ok(r) => r,
            Err(e) => {
                eprintln!("✗ Compile error (state preserved):\n{}", e);
                return;
            }
        };
        let new_screen = match compiled.ir.screens.first() {
            Some(s) => s.clone(),
            None => {
                eprintln!("✗ No screen found in .ui file");
                return;
            }
        };
        let reload = session.apply(new_screen);
        generation += 1;
        print_native_session(&session, generation, &native_reload_label(&reload, &session));
    });
}

fn watch_loop(path: &Path, last_mtime: &mut SystemTime, mut on_reload: impl FnMut()) {
    loop {
        thread::sleep(Duration::from_millis(POLL_MS));
        let mtime = file_mtime(path);
        if mtime <= *last_mtime {
            continue;
        }
        *last_mtime = mtime;
        on_reload();
    }
}

fn reload_label(reload: &flash_runtime::HotReloadResult, session: &DevSession) -> String {
    match reload.kind {
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
    }
}

fn native_reload_label(reload: &flash_runtime::HotReloadResult, session: &NativeDevSession) -> String {
    match reload.kind {
        HotReloadKind::HotReload => {
            format!(
                "Native hot reload — {} prop op(s), state preserved",
                reload.props_patched
            )
        }
        HotReloadKind::HotRestart => {
            format!(
                "Native hot restart — {} view(s), {} slot(s) preserved",
                session.view_count(),
                session.slot_values().len()
            )
        }
    }
}

fn print_mock_session(session: &DevSession, generation: u32, event: &str) {
    println!("── gen {} ── {}", generation, event);
    for line in session.renderer.describe() {
        println!("{}", line);
    }
    print_slots(session.slot_values());
}

fn print_native_session(session: &NativeDevSession, generation: u32, event: &str) {
    println!("── gen {} ── {}", generation, event);
    for line in session.describe_views() {
        println!("  {}", line);
    }
    print_slots(session.slot_values());
}

fn print_slots(slots: &[i64]) {
    if !slots.is_empty() {
        print!("   slots:");
        for (i, v) in slots.iter().enumerate() {
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
