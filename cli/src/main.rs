//! Flash CLI — mobile-first (iOS, Android) + Web
//!
//! ```bash
//! flash ir <file.ui>       # dump UI IR
//! flash run <file.ui>      # compile + simulate reactive update
//! flash platforms          # list supported targets
//! flash docs [--port 3000] # serve language documentation site
//! ```

use std::env;
use std::fs;
use std::io::{Read, Write};
use std::net::TcpListener;
use std::path::{Path, PathBuf};
use std::process;
use std::thread;

use flash_driver::compile;
use flash_ir::HandlerId;
use flash_platform::PlatformTarget;
use flash_runtime::{MockRenderer, PropValue, ReactiveEngine, RenderCall};

fn main() {
    let args: Vec<String> = env::args().collect();
    if args.len() < 2 {
        print_usage();
        process::exit(1);
    }

    match args[1].as_str() {
        "ir" => {
            let path = args.get(2).expect("usage: flash ir <file.ui>");
            let source = fs::read_to_string(path).expect("failed to read file");
            match compile(&source) {
                Ok(result) => println!("{}", result.ir_text),
                Err(e) => {
                    eprintln!("{}", e);
                    process::exit(1);
                }
            }
        }
        "run" => {
            let path = args.get(2).expect("usage: flash run <file.ui>");
            let source = fs::read_to_string(path).expect("failed to read file");
            run_simulation(&source);
        }
        "platforms" => {
            println!("Flash targets (mobile-first):\n");
            for target in PlatformTarget::all() {
                println!(
                    "  {:8}  {} via {}  (triple: {})",
                    target.name(),
                    target.native_backend(),
                    target.ffi_mechanism(),
                    target.triple(),
                );
            }
        }
        "docs" => {
            let port = parse_port_flag(&args).unwrap_or(3000);
            serve_docs(port);
        }
        "help" | "--help" | "-h" => print_usage(),
        cmd => {
            eprintln!("unknown command: {}", cmd);
            print_usage();
            process::exit(1);
        }
    }
}

fn run_simulation(source: &str) {
    let result = compile(source).expect("compilation failed");
    let screen = result.ir.screens.first().expect("no screen found").clone();

    let mut engine = ReactiveEngine::new(screen);
    let mut renderer = MockRenderer::new();

    engine.mount(&mut renderer);
    println!("=== Mounted ===");
    print_log(&renderer);
    renderer.clear_log();

    println!("\n=== Tap Increment (count++) ===");
    engine.fire_handler(HandlerId(0));
    engine.flush(&mut renderer);
    print_log(&renderer);

    // Verify thesis: exactly one SetProp + one Commit
    let set_props = renderer.log().iter()
        .filter(|c| matches!(c, RenderCall::SetProp { .. }))
        .count();
    let commits = renderer.log().iter()
        .filter(|c| matches!(c, RenderCall::Commit))
        .count();

    println!("\n=== Verification ===");
    println!("  SetProp calls: {} (expected: 1)", set_props);
    println!("  Commit calls:  {} (expected: 1)", commits);

    if set_props == 1 && commits == 1 {
        println!("  ✓ Fine-grained update confirmed — only Text node updated");
    } else {
        eprintln!("  ✗ Update count mismatch — full rebuild may have occurred");
        process::exit(1);
    }
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
        eprintln!("Error: site/ directory not found. Run from the Flash repo root.");
        process::exit(1);
    });
    println!("Flash language docs at http://localhost:{}/", port);
    println!("Serving from: {}", site.display());
    println!("Press Ctrl+C to stop.\n");

    let listener = TcpListener::bind(format!("127.0.0.1:{}", port))
        .unwrap_or_else(|e| {
            eprintln!("Failed to bind port {}: {}", port, e);
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
    eprintln!(
        "Flash — compiled cross-platform UI framework (iOS, Android, Web)\n\
         \n\
         Usage:\n\
           flash ir <file.ui>       Dump UI IR\n\
           flash run <file.ui>      Compile + simulate reactive update\n\
           flash platforms          List mobile/web targets\n\
           flash docs [--port N]    Serve language documentation site\n\
           flash help               Show this help"
    );
}
