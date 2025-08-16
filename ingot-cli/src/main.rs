use clap::{Parser, Subcommand};
use std::fs;
use std::process::Command;
use std::path::{Path, PathBuf};
use std::time::Duration;
use std::thread::sleep;
use std::fs::metadata;
use serde::Deserialize;
use std::env;

#[derive(Parser)]
#[command(name = "wingot")]
#[command(about = "The blazing-fast CLI for W++ WASM projects 🚀", long_about = None)]
struct Cli {
    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Initialize a new W++ project
    Init {
        #[arg(default_value = "my-wpp-project")]
        name: String,
    },
    /// Run a W++ file using the WASM runtime
    Run {
        #[arg(default_value = "", help = "Path to the W++ file (optional if wpp.json exists)")]
        file: String,

        #[arg(short, long, help = "Watch the file and rerun on changes")]
        watch: bool,
    },
}

#[derive(Deserialize)]
struct WppConfig {
    main: Option<String>,
    output: Option<String>,
    target: Option<String>, // <- new field to determine if wasm or native
}

fn load_config() -> Option<WppConfig> {
    if let Ok(contents) = fs::read_to_string("wpp.json") {
        serde_json::from_str(&contents).ok()
    } else {
        None
    }
}

/// Resolve the path to `wpp_wasm_runtime`
/// 1. Look in the same folder as the CLI binary
/// 2. Otherwise, assume it's in PATH
fn resolve_runtime() -> PathBuf {
    let exe_dir = env::current_exe()
        .ok()
        .and_then(|p| p.parent().map(|d| d.to_path_buf()))
        .unwrap_or_else(|| PathBuf::from("."));

    let local_path = exe_dir.join("wpp_wasm_runtime");

    if local_path.exists() {
        local_path
    } else {
        PathBuf::from("wpp_wasm_runtime") // fallback: rely on PATH
    }
}

fn main() {
    let cli = Cli::parse();

    match &cli.command {
        Commands::Init { name } => {
            let path = Path::new(name);
            if let Err(e) = fs::create_dir_all(path) {
                eprintln!("❌ Failed to create project folder: {e}");
                std::process::exit(1);
            }

            let wpp_json = format!(
r#"{{
  "name": "{}",
  "version": "1.0.0",
  "main": "main.wpp",
  "jit": false,
  "dependencies": {{}},
  "target": "wasm",
  "description": "A sample W++ project",
  "output": "wpp.wasm"
}}"#, name);

            if let Err(e) = fs::write(path.join("wpp.json"), wpp_json) {
                eprintln!("❌ Failed to write wpp.json: {e}");
                std::process::exit(1);
            }

            fs::write(path.join("main.wpp"), include_str!("../templates/main.wpp")).unwrap();
            fs::write(path.join("index.html"), include_str!("../templates/index.html")).unwrap();
            fs::write(path.join("wpp_loader.js"), include_str!("../templates/wpp_loader.js")).unwrap();

            println!("✅ Initialized W++ project: {}", name);
        }

        Commands::Run { file, watch } => {
            let config = load_config();

            let file_path = if file.is_empty() {
                config
                    .as_ref()
                    .and_then(|c| c.main.as_ref())
                    .cloned()
                    .unwrap_or_else(|| "main.wpp".to_string())
            } else {
                file.clone()
            };

            let output_file = config
                .as_ref()
                .and_then(|c| c.output.as_ref())
                .cloned()
                .unwrap_or_else(|| "wpp.wasm".to_string());

            let target = config
                .as_ref()
                .and_then(|c| c.target.as_ref())
                .map(|s| s.to_lowercase())
                .unwrap_or_else(|| "native".to_string());

            if !Path::new(&file_path).exists() {
                eprintln!("❌ File not found: {}", file_path);
                std::process::exit(1);
            }

            let mut last_modified = metadata(&file_path).unwrap().modified().unwrap();
            let runtime = resolve_runtime();

            loop {
                println!("⚙️ Compiling {} to WASM...", file_path);

                let status = Command::new(&runtime)
                    .arg(&file_path)
                    .arg("-o")
                    .arg(&output_file)
                    .status();

                match status {
                    Ok(s) if s.success() => {
                        if target == "native" {
                            println!("🚀 Running {} with Wasmtime...", output_file);
                            let run = Command::new("wasmtime")
                                .arg(&output_file)
                                .status();

                            match run {
                                Ok(r) if r.success() => {}
                                Ok(_) => eprintln!("❌ Runtime error."),
                                Err(e) => eprintln!("❌ Failed to launch Wasmtime: {e}"),
                            }
                        } else {
                            println!("🕸️ WebAssembly build complete – open index.html in a browser to run.");
                        }
                    }
                    Ok(_) => {
                        eprintln!("❌ Compilation failed (runtime returned error).");
                        if !watch {
                            break;
                        }
                    }
                    Err(e) => {
                        eprintln!("❌ Failed to run compiler: {e}");
                        std::process::exit(1);
                    }
                }

                if !watch {
                    break;
                }

                println!("🔄 Watching for changes to {}...", file_path);
                loop {
                    sleep(Duration::from_secs(1));
                    let new_time = metadata(&file_path).unwrap().modified().unwrap();
                    if new_time > last_modified {
                        last_modified = new_time;
                        println!("🔁 Change detected, re-running...");
                        break;
                    }
                }
            }
        }
    }
}
