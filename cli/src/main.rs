//! Flash CLI — Flutter-style developer experience.

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;

mod create;
mod dev;
mod devices;
mod doctor;
mod project;
mod ui;

use flash_driver::compile;
use flash_ir::HandlerId;
use flash_platform::{InProcessHost, PlatformTarget};
use flash_runtime::{MockRenderer, NativeSession, PropValue, ReactiveEngine, RenderCall};

use devices::Device;

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        show_dashboard();
        return;
    }

    match args[1].as_str() {
        "ir" => cmd_ir(&args),
        "run" => cmd_run(&args),
        "dev" => cmd_dev(&args),
        "build" => {
            let target = args.get(2).map(|s| s.as_str()).unwrap_or("ios");
            run_build(target);
        }
        "devices" => devices::list_devices(),
        "pub" => cmd_pub(&args),
        "platforms" => cmd_platforms(),
        "docs" => serve_docs(parse_port_flag(&args).unwrap_or(3000)),
        "doctor" | "setup" => doctor::run_doctor(),
        "create" => {
            let name = args.get(2).expect("usage: flash create <name> [ios|android|all]");
            let target = args.get(3).map(|s| s.as_str()).unwrap_or("all");
            create::run_create(name, target);
        }
        "help" | "--help" | "-h" => print_usage(),
        cmd => {
            ui::error(&format!("Unknown command: {}", cmd));
            print_usage();
            process::exit(1);
        }
    }
}

fn show_dashboard() {
    if let Some(proj) = project::load() {
        ui::project_welcome(&proj.name, &proj.entry.display().to_string());
    } else {
        ui::welcome();
    }
}

/// `flash run` — hot reload by default (like `flutter run`).
fn cmd_run(args: &[String]) {
    let opts = parse_run_opts(args);
    if opts.once {
        run_once(opts.path.as_deref().map(Path::new), opts.device);
    } else {
        dev::run_dev(opts.path.as_deref().map(Path::new), opts.device);
    }
}

/// `flash dev` — alias for `flash run`.
fn cmd_dev(args: &[String]) {
    let opts = parse_run_opts(args);
    dev::run_dev(opts.path.as_deref().map(Path::new), opts.device);
}

fn cmd_pub(args: &[String]) {
    let sub = args.get(2).map(|s| s.as_str()).unwrap_or("get");
    match sub {
        "get" => project::pub_get(),
        other => {
            ui::error(&format!("Unknown pub subcommand: {}", other));
            process::exit(1);
        }
    }
}

fn cmd_ir(args: &[String]) {
    let path = project::resolve_ui(args.get(2).map(Path::new));
    let source = fs::read_to_string(&path).expect("failed to read file");
    match compile(&source) {
        Ok(result) => println!("{}", result.ir_text),
        Err(e) => {
            ui::error(&e.to_string());
            process::exit(1);
        }
    }
}

fn cmd_platforms() {
    ui::banner();
    println!();
    for target in PlatformTarget::all() {
        println!(
            "  {:8}  {} via {}",
            target.name(),
            target.native_backend(),
            target.ffi_mechanism(),
        );
    }
    println!();
}

struct RunOpts {
    path: Option<String>,
    device: Device,
    once: bool,
}

fn parse_run_opts(args: &[String]) -> RunOpts {
    let mut path = None;
    let mut device = Device::default();
    let mut once = false;

    let mut i = 2;
    while i < args.len() {
        match args[i].as_str() {
            "-d" | "--device" => {
                i += 1;
                let id = args.get(i).expect("usage: -d <device> (simulator|native|ios|android)");
                device = Device::parse(id).unwrap_or_else(|| {
                    ui::error(&format!("Unknown device '{}'. Run flash devices.", id));
                    process::exit(1);
                });
            }
            "--once" => once = true,
            "--native" => device = Device::Native,
            "ios" if path.is_none() => device = Device::Ios,
            "android" if path.is_none() => device = Device::Android,
            s if !s.starts_with('-') && path.is_none() => path = Some(s.to_string()),
            _ => {}
        }
        i += 1;
    }

    RunOpts { path, device, once }
}

fn run_once(path: Option<&Path>, device: Device) {
    let path = project::resolve_ui(path);
    let source = fs::read_to_string(&path).expect("failed to read file");
    ui::launch_line(&path.display().to_string(), device.label());

    match device {
        Device::Simulator => run_simulation(&source),
        Device::Native | Device::Ios | Device::Android => {
            let platform = match device {
                Device::Ios => Some("ios"),
                Device::Android => Some("android"),
                _ => None,
            };
            run_native(&source, platform);
        }
    }
}

fn run_native(source: &str, platform: Option<&str>) {
    let result = compile(source).expect("compilation failed");
    let screen = result.ir.screens.first().cloned().expect("no screen found");

    let host = InProcessHost::default();
    let mut session = NativeSession::mount(screen, Box::new(host));

    if let Some(p) = platform {
        ui::step(&format!("Native render path: {}", p));
        ui::dim("  Open platform/ios or platform/android for on-device builds");
    } else {
        ui::success("Native command buffer mounted");
    }

    println!();
    println!("  slots: {:?}", session.engine.slots.ints);
    ui::step("Simulating tap (handler 0)...");
    session.fire_handler(HandlerId(0));
    println!("  slots: {:?}", session.engine.slots.ints);
    ui::success("Done.");
    println!();
}

fn run_build(target: &str) {
    ui::banner();
    println!();
    match target {
        "ios" => {
            ui::step("Building for iOS...");
            println!("  1. ./scripts/build-rust.sh");
            println!("  2. open platform/ios/FlashApp.xcodeproj");
            println!("  3. Run on simulator or device");
        }
        "android" => {
            ui::step("Building for Android...");
            println!("  1. ./scripts/build-rust.sh");
            println!("  2. open platform/android in Android Studio");
            println!("  3. Run on emulator or device");
        }
        other => {
            ui::error(&format!("Unknown target: {} (use ios or android)", other));
            process::exit(1);
        }
    }
    println!();
}

fn run_simulation(source: &str) {
    let result = compile(source).expect("compilation failed");
    let screen = result.ir.screens.first().expect("no screen found").clone();

    let mut engine = ReactiveEngine::new(screen);
    let mut renderer = MockRenderer::new();

    engine.mount(&mut renderer);
    ui::success("Mounted");
    print_log(&renderer);
    renderer.clear_log();

    ui::step("Tap handler 0 (count++)");
    engine.fire_handler(HandlerId(0));
    engine.flush(&mut renderer);
    print_log(&renderer);

    let set_props = renderer
        .log()
        .iter()
        .filter(|c| matches!(c, RenderCall::SetProp { .. }))
        .count();
    let commits = renderer
        .log()
        .iter()
        .filter(|c| matches!(c, RenderCall::Commit))
        .count();

    println!();
    if set_props == 1 && commits == 1 {
        ui::success("Fine-grained update — 1 SetProp + 1 Commit");
    } else {
        ui::warn(&format!("SetProp={}, Commit={} (expected 1 each)", set_props, commits));
    }
    println!();
}

fn print_log(renderer: &MockRenderer) {
    for call in renderer.log() {
        match call {
            RenderCall::Create { kind } => println!("  CREATE {}", kind.name()),
            RenderCall::SetProp { node, key, value } => {
                let val = match value {
                    PropValue::Str(s) => s.clone(),
                    PropValue::Int(v) => v.to_string(),
                    PropValue::Float(v) => v.to_string(),
                    PropValue::Bool(v) => v.to_string(),
                };
                println!("  SET_PROP node={} key={:?} value=\"{}\"", node.0, key, val);
            }
            RenderCall::Commit => println!("  COMMIT"),
        }
    }
}

fn parse_port_flag(args: &[String]) -> Option<u16> {
    for (i, arg) in args.iter().enumerate() {
        if arg == "--port" || arg == "-p" {
            return args.get(i + 1).and_then(|s| s.parse().ok());
        }
        if let Some(p) = arg.strip_prefix("--port=") {
            return p.parse().ok();
        }
    }
    None
}

fn serve_docs(port: u16) {
    let site = find_site_dir().unwrap_or_else(|| {
        ui::error("site/ directory not found. Run from the Flash repo root.");
        process::exit(1);
    });
    ui::banner();
    println!();
    ui::success(&format!("Docs at http://localhost:{}/", port));
    ui::dim(&format!("Serving {}", site.display()));
    ui::dim("Press Ctrl+C to stop.");
    println!();

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .unwrap_or_else(|e| {
            ui::error(&format!("Failed to bind port {}: {}", port, e));
            process::exit(1);
        });

    for stream in listener.incoming().flatten() {
        let site = site.clone();
        thread::spawn(move || handle_http_request(stream, &site));
    }
}

fn handle_http_request(mut stream: std::net::TcpStream, site: &Path) {
    let mut buf = [0u8; 4096];
    if stream.read(&mut buf).is_err() {
        return;
    }
    let req = String::from_utf8_lossy(&buf);
    let path = req
        .lines()
        .next()
        .and_then(|l| l.split_whitespace().nth(1))
        .unwrap_or("/");
    let file_path = resolve_site_path(site, path);
    let (status, content_type, body) = if file_path.is_file() {
        let body = fs::read(&file_path).unwrap_or_default();
        let ct = content_type_for(&file_path);
        ("200 OK", ct, body)
    } else {
        ("404 Not Found", "text/plain", b"Not found".to_vec())
    };
    let resp = format!(
        "HTTP/1.1 {}\r\nContent-Type: {}\r\nContent-Length: {}\r\nConnection: close\r\n\r\n",
        status,
        content_type,
        body.len()
    );
    let _ = stream.write_all(resp.as_bytes());
    let _ = stream.write_all(&body);
}

fn find_site_dir() -> Option<PathBuf> {
    for candidate in [PathBuf::from("site"), PathBuf::from("../site")] {
        if candidate.join("index.html").is_file() {
            return Some(candidate.canonicalize().unwrap_or(candidate));
        }
    }
    None
}

fn resolve_site_path(site: &Path, url_path: &str) -> PathBuf {
    let clean = url_path.trim_start_matches('/');
    if clean.is_empty() {
        return site.join("index.html");
    }
    let p = site.join(clean);
    if p.is_file() {
        return p;
    }
    let with_html = site.join(format!("{}.html", clean.trim_end_matches('/')));
    if with_html.is_file() {
        return with_html;
    }
    site.join("index.html")
}

fn content_type_for(path: &Path) -> &'static str {
    match path.extension().and_then(|e| e.to_str()) {
        Some("html") => "text/html; charset=utf-8",
        Some("css") => "text/css",
        Some("js") => "application/javascript",
        Some("json") => "application/json",
        Some("svg") => "image/svg+xml",
        Some("png") => "image/png",
        _ => "application/octet-stream",
    }
}

fn print_usage() {
    ui::banner();
    eprintln!(
        "\n\
         Usage:\n\
           flash                    Project dashboard or welcome\n\
           flash create <name>      New app (clean architecture)\n\
           flash run                Hot reload (default, like flutter run)\n\
           flash run -d native      Native command-buffer hot reload\n\
           flash run --once         One-shot simulate\n\
           flash devices            List run targets\n\
           flash pub get            Fetch Rust workspace deps\n\
           flash doctor             Verify toolchain\n\
           flash build [ios|android]  Platform build guide\n\
           flash docs [--port N]    Serve documentation site\n\
           flash ir [file.ui]       Dump UI IR\n\
         \n\
         Install:\n\
           curl -fsSL https://raw.githubusercontent.com/Anu-Code07/flash/main/install.sh | bash\n"
    );
}
