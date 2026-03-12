use std::path::Path;
use std::process::Command;
use std::{env, process};

use fs_err as fs;

fn main() {
    let args: Vec<String> = env::args().collect();

    match args.get(1).map(|s| s.as_str()) {
        Some("new") => {
            if args.len() != 3 {
                eprintln!("Usage: ui-tests new <name>");
                process::exit(1);
            }
            cmd_new(&args[2]);
        }
        Some("translate-configs") => {
            if args.len() != 2 {
                eprintln!("Usage: ui-tests translate-configs");
                process::exit(1);
            }
            cmd_translate_configs();
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  ui-tests new <name>            Create a new cheadergen test case");
            eprintln!(
                "  ui-tests translate-configs     Translate all cbindgen.toml files to cheadergen.toml"
            );
            process::exit(1);
        }
    }
}

fn cmd_new(name: &str) {
    if !is_valid_crate_name(name) {
        eprintln!(
            "Error: '{name}' is not a valid crate name (must be alphanumeric + underscore, cannot start with a digit)"
        );
        process::exit(1);
    }

    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let cheadergen_case = tests_dir.join("cheadergen/rust/cases").join(name);
    let cbindgen_case = tests_dir.join("cbindgen/rust/cases").join(name);

    if cheadergen_case.exists() {
        eprintln!("Error: cheadergen test case '{name}' already exists");
        process::exit(1);
    }
    if cbindgen_case.exists() {
        eprintln!(
            "Error: cbindgen test case '{name}' already exists — pick a different name to avoid confusion"
        );
        process::exit(1);
    }

    let src_dir = cheadergen_case.join("src");
    fs::create_dir_all(&src_dir).unwrap_or_else(|e| {
        eprintln!("Error: failed to create directory: {e}");
        process::exit(1);
    });

    let cargo_toml = format!(
        "\
[package]
name = \"{name}\"
version = \"0.1.0\"
edition.workspace = true
"
    );

    let lib_rs = "\
#[repr(C)]
pub struct TODO {
    pub field: u32,
}

#[unsafe(no_mangle)]
pub extern \"C\" fn todo_new() -> TODO {
    TODO { field: 0 }
}
";

    fs::write(cheadergen_case.join("Cargo.toml"), cargo_toml).unwrap_or_else(|e| {
        eprintln!("Error: failed to write Cargo.toml: {e}");
        process::exit(1);
    });
    fs::write(src_dir.join("lib.rs"), lib_rs).unwrap_or_else(|e| {
        eprintln!("Error: failed to write src/lib.rs: {e}");
        process::exit(1);
    });
    fs::write(cheadergen_case.join("test.toml"), "").unwrap_or_else(|e| {
        eprintln!("Error: failed to write test.toml: {e}");
        process::exit(1);
    });

    println!("Created test case at {}", cheadergen_case.display());
    println!();
    println!("Next steps:");
    println!(
        "  1. Edit {}/src/lib.rs with your test code",
        cheadergen_case.display()
    );
    println!("  2. Run `just test-generate` to generate expectation snapshots");
    println!("  3. Review and accept the snapshots");
}

fn cmd_translate_configs() {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let cbindgen_rust_dir = tests_dir.join("cbindgen/rust");

    let mut configs = Vec::new();
    collect_cbindgen_tomls(&cbindgen_rust_dir, &mut configs);
    configs.sort();

    if configs.is_empty() {
        println!("No cbindgen.toml files found.");
        return;
    }

    println!("Found {} cbindgen.toml file(s)\n", configs.len());

    let build = Command::new("cargo")
        .args(["build", "-p", "cheadergen", "--quiet"])
        .status();
    match build {
        Ok(s) if s.success() => {}
        Ok(s) => {
            eprintln!("Failed to build cheadergen (exit code: {})", s);
            process::exit(1);
        }
        Err(e) => {
            eprintln!("Failed to run cargo build: {e}");
            process::exit(1);
        }
    }

    let cheadergen_bin = Path::new(env!("CARGO_MANIFEST_DIR")).join("../target/debug/cheadergen");

    let mut failures = Vec::new();
    let mut modified = 0u32;

    for config in &configs {
        let parent = config.parent().unwrap();
        let output = parent.join("cheadergen.toml");

        let old_contents = fs::read(&output).ok();

        let result = Command::new(&cheadergen_bin)
            .args(["config", "translate", "--from"])
            .arg(config)
            .arg("--to")
            .arg(&output)
            .output();

        match result {
            Ok(out) if out.status.success() => {
                if old_contents.as_deref() != fs::read(&output).ok().as_deref() {
                    println!("MODIFIED: {}", config.display());
                    modified += 1;
                }
            }
            Ok(out) => {
                let stderr = String::from_utf8_lossy(&out.stderr);
                eprintln!("FAIL: {}", config.display());
                eprintln!("      {}", stderr.trim());
                failures.push(config.clone());
            }
            Err(e) => {
                eprintln!("FAIL: {}", config.display());
                eprintln!("      {e}");
                failures.push(config.clone());
            }
        }
    }

    println!();
    if !failures.is_empty() {
        eprintln!(
            "{}/{} translation(s) failed:",
            failures.len(),
            configs.len()
        );
        for f in &failures {
            eprintln!("  - {}", f.display());
        }
        process::exit(1);
    } else if modified == 0 {
        println!(
            "No configuration changes after translating {} cbindgen.toml file(s).",
            configs.len()
        );
    } else {
        println!("{modified}/{} config(s) modified.", configs.len());
    }
}

fn collect_cbindgen_tomls(dir: &Path, out: &mut Vec<std::path::PathBuf>) {
    let entries = match fs::read_dir(dir) {
        Ok(entries) => entries,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_cbindgen_tomls(&path, out);
        } else if path.file_name().is_some_and(|n| n == "cbindgen.toml") {
            out.push(path);
        }
    }
}

fn is_valid_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
