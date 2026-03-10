use std::path::Path;
use std::{env, fs, process};

fn main() {
    let args: Vec<String> = env::args().collect();

    if args.len() != 3 || args[1] != "new" {
        eprintln!("Usage: ui-tests new <name>");
        process::exit(1);
    }

    let name = &args[2];

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

fn is_valid_crate_name(name: &str) -> bool {
    !name.is_empty()
        && name
            .chars()
            .next()
            .is_some_and(|c| c.is_ascii_alphabetic() || c == '_')
        && name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
}
