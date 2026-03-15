use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, process};

use fs_err as fs;

const VARIANTS: &[&str] = &[
    "c/plain",
    "c/tag",
    "c/both",
    "c/compat",
    "c/tag_compat",
    "c/both_compat",
    "cpp/plain",
    "cython/plain",
    "cython/tag",
    "symbol",
];

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
                for v in VARIANTS {
                    eprintln!("  {v}");
                }
                process::exit(1);
            });
            if !VARIANTS.contains(&variant) {
                eprintln!("Unknown variant: {variant}");
                eprintln!();
                eprintln!("Valid variants:");
                for v in VARIANTS {
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

fn parse_test_toml(path: &Path) -> BTreeMap<String, String> {
    let content = match fs::read_to_string(path) {
        Ok(c) => c,
        Err(_) => return BTreeMap::new(),
    };
    let mut map = BTreeMap::new();
    for line in content.lines() {
        let line = line.trim();
        // Each line is "key" = "value"
        if let Some(rest) = line.strip_prefix('"')
            && let Some(eq) = rest.find("\" = \"")
        {
            let key = &rest[..eq];
            let value_start = eq + "\" = \"".len();
            if let Some(value) = rest[value_start..].strip_suffix('"') {
                map.insert(key.to_owned(), value.to_owned());
            }
        }
    }
    map
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
    let mut cases = Vec::new();

    let cases_dir = cbindgen_rust_dir.join("cases");
    if let Ok(entries) = fs::read_dir(&cases_dir) {
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() && path.join("test.toml").exists() {
                cases.push(path);
            }
        }
    }

    for extra in &["workspace", "external_workspace_child"] {
        let path = cbindgen_rust_dir.join(extra);
        if path.join("test.toml").exists() {
            cases.push(path);
        }
    }

    cases.sort();
    cases
}

fn cmd_cbindgen_report(variant: &str) {
    let tests_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("tests");
    let cbindgen_rust_dir = tests_dir.join("cbindgen/rust");
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
        let test_toml = parse_test_toml(&case_path.join("test.toml"));

        let status = test_toml
            .get(variant)
            .map(|s| s.as_str())
            .unwrap_or("normal");

        match status {
            "xfail" => {
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
            "normal" => normal_cases.push(case_name),
            "skip" => skip_cases.push(case_name),
            "exclude" => exclude_cases.push(case_name),
            _ => normal_cases.push(case_name),
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
    let manifest_path = tests_dir.join("cheadergen/.test_manifest");
    let mut case_names: Vec<String> = fs::read_to_string(&manifest_path)
        .unwrap_or_default()
        .lines()
        .filter(|l| !l.is_empty())
        .map(|l| l.to_owned())
        .collect();
    if !case_names.contains(&name.to_owned()) {
        case_names.push(name.to_owned());
    }
    case_names.sort();
    let new_manifest = case_names.join("\n") + "\n";
    fs::write(&manifest_path, new_manifest).unwrap_or_else(|e| {
        eprintln!("Warning: failed to update .test_manifest: {e}");
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
        let summary_path = tests_dir.join("cbindgen/unsupported_keys.txt");
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
