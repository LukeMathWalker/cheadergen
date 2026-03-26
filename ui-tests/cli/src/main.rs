use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, process};

use fs_err as fs;
use ui_tests_toolkit::{
    VariantStatus, collect_case_dirs, read_manifest_file, read_test_manifest, variant_path_strings,
    write_manifest_file,
};

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
        Some("cbindgen-report") => {
            let variant = args
                .iter()
                .skip(2)
                .find(|a| !a.starts_with('-'))
                .cloned();
            let variant = variant.as_deref().unwrap_or_else(|| {
                eprintln!("Usage: ui-tests cbindgen-report <variant>");
                eprintln!();
                eprintln!("Variants:");
                for v in variant_path_strings() {
                    eprintln!("  {v}");
                }
                process::exit(1);
            });
            if !variant_path_strings().iter().any(|v| v == variant) {
                eprintln!("Unknown variant: {variant}");
                eprintln!();
                eprintln!("Valid variants:");
                for v in variant_path_strings() {
                    eprintln!("  {v}");
                }
                process::exit(1);
            }
            cmd_cbindgen_report(variant);
        }
        _ => {
            eprintln!("Usage:");
            eprintln!("  ui-tests new <name>            Create a new cheadergen test case");
            eprintln!(
                "  ui-tests translate-configs     Translate all cbindgen.toml files to cheadergen.toml"
            );
            eprintln!(
                "  ui-tests cbindgen-report <variant>  Print cbindgen compatibility report for a variant"
            );
            process::exit(1);
        }
    }
}

fn detect_cbindgen_annotations(case_dir: &Path) -> bool {
    let src_dir = case_dir.join("src");
    let entries = match fs::read_dir(&src_dir) {
        Ok(e) => e,
        Err(_) => return false,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.extension().is_some_and(|e| e == "rs")
            && path.is_file()
            && let Ok(content) = fs::read_to_string(&path)
        {
            for line in content.lines() {
                let trimmed = line.trim_start();
                if trimmed.starts_with("/// cbindgen:") || trimmed.starts_with("// cbindgen:") {
                    return true;
                }
            }
        }
    }
    false
}

fn collect_cbindgen_cases(cbindgen_rust_dir: &Path) -> Vec<PathBuf> {
    let cases_dir = cbindgen_rust_dir.join("cases");
    let mut cases: Vec<PathBuf> = collect_case_dirs(&cases_dir)
        .into_iter()
        .map(|name| cases_dir.join(name))
        .collect();

    for extra in &["workspace", "external_workspace_child"] {
        let path = cbindgen_rust_dir.join(extra);
        if path.join("test.toml").exists() {
            cases.push(path);
        }
    }

    cases
}

fn workspace_root() -> &'static Path {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .and_then(|p| p.parent())
        .expect("ui-tests must be inside a workspace")
}

fn cmd_cbindgen_report(variant: &str) {
    let cbindgen_rust_dir = workspace_root().join("ui-tests/cbindgen/tests/cbindgen/rust");
    let cases = collect_cbindgen_cases(&cbindgen_rust_dir);

    let mut xfail_with_annotations: Vec<String> = Vec::new();
    let mut xfail_with_unsupported: Vec<String> = Vec::new();
    let mut xfail_neither: Vec<String> = Vec::new();
    let mut normal_cases: Vec<String> = Vec::new();
    let mut skip_cases: Vec<String> = Vec::new();
    let mut exclude_cases: Vec<String> = Vec::new();
    let mut unsupported_keys: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for case_path in &cases {
        let case_name = case_path
            .file_name()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        let toml_path = case_path.join("test.toml");
        let test_toml = read_test_manifest(&toml_path).unwrap_or_default();

        let status = test_toml.get(variant).copied();

        match status {
            Some(VariantStatus::HeaderDiff | VariantStatus::GenerationFails) => {
                let has_annotations = detect_cbindgen_annotations(case_path);
                let skipped_fields = extract_skipped_fields(&case_path.join("cheadergen.toml"));
                let has_unsupported = !skipped_fields.is_empty();

                for field in skipped_fields {
                    unsupported_keys
                        .entry(field)
                        .or_default()
                        .push(case_name.clone());
                }

                if has_annotations {
                    xfail_with_annotations.push(case_name);
                } else if has_unsupported {
                    xfail_with_unsupported.push(case_name);
                } else {
                    xfail_neither.push(case_name);
                }
            }
            None => normal_cases.push(case_name),
            Some(VariantStatus::Skip) => skip_cases.push(case_name),
            Some(VariantStatus::Exclude) => exclude_cases.push(case_name),
        }
    }

    let total_xfail =
        xfail_with_annotations.len() + xfail_with_unsupported.len() + xfail_neither.len();

    println!("cbindgen compatibility report: {variant}");
    println!("{}", "=".repeat(31 + variant.len()));
    println!("{} test cases", cases.len());
    println!();
    println!(
        "  Xfail:   {total_xfail:>4}    Normal: {:>4}    Skip: {:>4}    Exclude: {:>4}",
        normal_cases.len(),
        skip_cases.len(),
        exclude_cases.len()
    );
    println!();

    println!("--- Xfail breakdown ({total_xfail} cases) ---");
    println!();
    println!(
        "  Has cbindgen annotations:    {:>4}",
        xfail_with_annotations.len()
    );
    for name in &xfail_with_annotations {
        println!("    {name}");
    }
    println!(
        "  Has unsupported config keys:  {:>3}",
        xfail_with_unsupported.len()
    );
    for name in &xfail_with_unsupported {
        println!("    {name}");
    }
    println!(
        "  Neither (pure generation):   {:>4}",
        xfail_neither.len()
    );
    for name in &xfail_neither {
        println!("    {name}");
    }

    if !unsupported_keys.is_empty() {
        let unsupported_case_set: HashSet<&str> = unsupported_keys
            .values()
            .flat_map(|v| v.iter().map(|s| s.as_str()))
            .collect();

        println!();
        println!(
            "--- Unsupported config keys ({} fields across {} cases) ---",
            unsupported_keys.len(),
            unsupported_case_set.len()
        );
        println!();

        let max_key_len = unsupported_keys.keys().map(|k| k.len()).max().unwrap_or(0);
        for (key, cases) in &unsupported_keys {
            println!(
                "  {:<width$}  Cases ({}): {}",
                key,
                cases.len(),
                cases.join(", "),
                width = max_key_len
            );
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

    let root = workspace_root();
    let cheadergen_case = root.join("ui-tests/cheadergen/tests/cheadergen/rust/cases").join(name);
    let cbindgen_case = root.join("ui-tests/cbindgen/tests/cbindgen/rust/cases").join(name);

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

[lints]
workspace = true
"
    );

    let lib_rs = "\
//! TODO: Describe what this test case checks.

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
    fs::write(
        cheadergen_case.join("test.toml"),
        "\"cpp/plain\" = \"exclude\"\n\
         \"cython/plain\" = \"exclude\"\n\
         \"cython/tag\" = \"exclude\"\n",
    )
    .unwrap_or_else(|e| {
        eprintln!("Error: failed to write test.toml: {e}");
        process::exit(1);
    });

    // Update cheadergen .test_manifest so cargo picks up the new case immediately.
    let manifest_path = root.join("ui-tests/cheadergen/tests/cheadergen/.test_manifest");
    let mut case_names = read_manifest_file(&manifest_path);
    if !case_names.contains(&name.to_owned()) {
        case_names.push(name.to_owned());
    }
    case_names.sort();
    write_manifest_file(&manifest_path, &case_names);

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
    let root = workspace_root();
    let cbindgen_rust_dir = root.join("ui-tests/cbindgen/tests/cbindgen/rust");

    let mut configs = Vec::new();
    collect_cbindgen_tomls(&cbindgen_rust_dir, &mut configs);
    configs.sort();

    if configs.is_empty() {
        println!("No cbindgen.toml files found.");
        return;
    }

    println!("Found {} cbindgen.toml file(s)\n", configs.len());

    let build = Command::new("cargo")
        .args(["build", "-p", "cheadergen_cli", "--quiet"])
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

    let cheadergen_bin = root.join("target/debug/cheadergen");

    let mut failures = Vec::new();
    let mut modified = 0u32;
    let mut unsupported: BTreeMap<String, Vec<String>> = BTreeMap::new();

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
                let test_name = config
                    .parent()
                    .unwrap()
                    .file_name()
                    .unwrap()
                    .to_string_lossy()
                    .into_owned();
                for field in extract_skipped_fields(&output) {
                    unsupported
                        .entry(field)
                        .or_default()
                        .push(test_name.clone());
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

    // Write unsupported keys summary
    if !unsupported.is_empty() {
        let mut lines = Vec::new();
        for (field, mut tests) in unsupported {
            tests.sort();
            lines.push(format!("{}: {}", field, tests.join(", ")));
        }
        let summary_path = cbindgen_rust_dir.join("../unsupported_keys.txt");
        fs::write(&summary_path, lines.join("\n") + "\n").unwrap_or_else(|e| {
            eprintln!("Warning: failed to write unsupported keys summary: {e}");
        });
        println!(
            "Wrote unsupported keys summary to {}",
            summary_path.display()
        );
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

fn extract_skipped_fields(path: &Path) -> Vec<String> {
    let contents = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return Vec::new(),
    };
    let mut fields = Vec::new();
    for line in contents.lines() {
        // Match lines like: # `[export]` was skipped: not supported by cheadergen
        //                or: # `line_endings` was skipped: not supported by cheadergen
        let Some(rest) = line.strip_prefix("# `") else {
            continue;
        };
        let Some(rest) = rest.strip_suffix("` was skipped: not supported by cheadergen") else {
            continue;
        };
        fields.push(rest.to_owned());
    }
    fields
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
