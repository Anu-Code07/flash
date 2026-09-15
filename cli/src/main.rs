//! Flash CLI — mobile-first (iOS, Android) + Web
//!
//! ```bash
//! flash ir <file.ui>       # dump UI IR
//! flash run <file.ui>      # compile + simulate reactive update
//! flash platforms          # list supported targets
//! ```

use std::env;
use std::fs;
use std::process;

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

fn print_usage() {
    eprintln!(
        "Flash — compiled cross-platform UI framework (iOS, Android, Web)\n\
         \n\
         Usage:\n\
           flash ir <file.ui>       Dump UI IR\n\
           flash run <file.ui>      Compile + simulate reactive update\n\
           flash platforms          List mobile/web targets\n\
           flash help               Show this help"
    );
}
