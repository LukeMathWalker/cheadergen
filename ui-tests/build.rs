/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::{BTreeMap, HashSet};
use std::env;
use std::fs::{self, File};
use std::io::Write;
use std::path::{Path, PathBuf};

struct Variant {
    module_path: &'static [&'static str],
    lang: &'static str,
    style: &'static str,
    cpp_compat: bool,
    file_pattern: &'static str,
}

const VARIANTS: &[Variant] = &[
    Variant {
        module_path: &["c", "plain"],
        lang: "Language::C",
        style: "Some(Style::Type)",
        cpp_compat: false,
        file_pattern: "{name}.c",
    },
    Variant {
        module_path: &["c", "tag"],
        lang: "Language::C",
        style: "Some(Style::Tag)",
        cpp_compat: false,
        file_pattern: "{name}_tag.c",
    },
    Variant {
        module_path: &["c", "both"],
        lang: "Language::C",
        style: "Some(Style::Both)",
        cpp_compat: false,
        file_pattern: "{name}_both.c",
    },
    Variant {
        module_path: &["c", "compat"],
        lang: "Language::C",
        style: "Some(Style::Type)",
        cpp_compat: true,
        file_pattern: "{name}.compat.c",
    },
    Variant {
        module_path: &["c", "tag_compat"],
        lang: "Language::C",
        style: "Some(Style::Tag)",
        cpp_compat: true,
        file_pattern: "{name}_tag.compat.c",
    },
    Variant {
        module_path: &["c", "both_compat"],
        lang: "Language::C",
        style: "Some(Style::Both)",
        cpp_compat: true,
        file_pattern: "{name}_both.compat.c",
    },
    Variant {
        module_path: &["cpp", "plain"],
        lang: "Language::Cxx",
        style: "None",
        cpp_compat: false,
        file_pattern: "{name}.cpp",
    },
    Variant {
        module_path: &["cython", "plain"],
        lang: "Language::Cython",
        style: "Some(Style::Type)",
        cpp_compat: false,
        file_pattern: "{name}.pyx",
    },
    Variant {
        module_path: &["cython", "tag"],
        lang: "Language::Cython",
        style: "Some(Style::Tag)",
        cpp_compat: false,
        file_pattern: "{name}_tag.pyx",
    },
];

// Tree node for building nested module structure.
struct ModNode {
    children: BTreeMap<String, ModNode>,
    test_lines: Vec<String>,
}

impl ModNode {
    fn new() -> Self {
        ModNode {
            children: BTreeMap::new(),
            test_lines: Vec::new(),
        }
    }

    fn insert(&mut self, path: &[&str], line: String) {
        if path.is_empty() {
            self.test_lines.push(line);
        } else {
            self.children
                .entry(path[0].to_owned())
                .or_insert_with(ModNode::new)
                .insert(&path[1..], line);
        }
    }

    fn emit(&self, dst: &mut impl Write, depth: usize) {
        let indent = "    ".repeat(depth);
        for (name, child) in &self.children {
            writeln!(dst, "{indent}mod {name} {{").unwrap();
            if child.children.is_empty() && !child.test_lines.is_empty() {
                // Leaf module — emit use statement to bring root items into scope.
                let supers = "super::".repeat(depth + 1);
                writeln!(dst, "{indent}    use {supers}*;").unwrap();
            }
            child.emit(dst, depth + 1);
            for line in &child.test_lines {
                writeln!(dst, "{indent}    {line}").unwrap();
            }
            writeln!(dst, "{indent}}}").unwrap();
        }
    }
}

fn collect_variants(
    root: &mut ModNode,
    variants: &[Variant],
    suite: &str,
    expectations_dir: &Path,
    path_segment: &str,
    case_path: &Path,
    xfail: &HashSet<String>,
) {
    let base_name = path_segment
        .strip_suffix(".skip_warning_as_error")
        .unwrap_or(path_segment);

    let identifier_base = path_segment
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .replace("__", "_");

    let skip_warning_as_error = path_segment.contains(".skip_warning_as_error");

    let is_linestyle = path_segment.starts_with("linestyle_");

    for variant in variants {
        let expectation_file = variant.file_pattern.replace("{name}", base_name);
        let variant_path = variant.module_path.join("/");

        // Linestyle tests keep raw files; everything else uses .snap files.
        let resolved_path = if is_linestyle {
            let p = expectations_dir.join(&expectation_file);
            if p.exists() { Some(p) } else { None }
        } else {
            let snap = expectations_dir.join(format!("{expectation_file}.snap"));
            if snap.exists() { Some(snap) } else { None }
        };

        if let Some(resolved_path) = resolved_path {
            let xfail_key = format!("{} {}", path_segment, variant_path);
            let xfail_token = if xfail.contains(&xfail_key) {
                "xfail, "
            } else {
                ""
            };

            // Generate test
            let gen_line = format!(
                "generate_variant!({xfail_token}r#{}, {:?}, {:?}, {:?}, {}, {}, {});",
                identifier_base,
                path_segment,
                variant_path,
                case_path,
                variant.lang,
                variant.style,
                variant.cpp_compat,
            );
            let mut gen_path: Vec<&str> = vec![suite, "generate"];
            gen_path.extend_from_slice(variant.module_path);
            root.insert(&gen_path, gen_line);

            // Compile test
            let compile_line = format!(
                "compile_variant!({xfail_token}r#{}, {:?}, {:?}, {:?}, {}, {}, {}, {});",
                identifier_base,
                path_segment,
                variant_path,
                resolved_path,
                variant.lang,
                variant.style,
                skip_warning_as_error,
                variant.cpp_compat,
            );
            let mut compile_path: Vec<&str> = vec![suite, "compile"];
            compile_path.extend_from_slice(variant.module_path);
            root.insert(&compile_path, compile_line);
        }
    }
}

struct TestSuite<'a> {
    name: &'a str,
    cases_dir: PathBuf,
    expectations_dir: PathBuf,
    extra_dirs: Vec<PathBuf>,
    manifest_path: Option<PathBuf>,
}

fn process_suite(
    suite: &TestSuite,
    dst: &mut File,
    root: &mut ModNode,
    variants: &[Variant],
    const_name: &str,
    xfail: &HashSet<String>,
) -> Vec<String> {
    // Watch the cases workspace definition.
    println!(
        "cargo:rerun-if-changed={}",
        suite.cases_dir.join("Cargo.toml").display()
    );

    // Watch the expectations directory so new/removed files trigger regeneration.
    println!(
        "cargo:rerun-if-changed={}",
        suite.expectations_dir.display()
    );

    let mut case_names: Vec<String> = Vec::new();

    for entry in fs::read_dir(&suite.cases_dir).unwrap() {
        let entry = entry.expect("Couldn't read test entry");

        if !entry.file_type().unwrap().is_dir() {
            continue;
        }

        let path_segment = entry.file_name().to_str().unwrap().to_owned();

        // Watch each crate's Cargo.toml for identity changes.
        println!(
            "cargo:rerun-if-changed={}",
            entry.path().join("Cargo.toml").display()
        );

        collect_variants(
            root,
            variants,
            suite.name,
            &suite.expectations_dir,
            &path_segment,
            &entry.path(),
            xfail,
        );

        case_names.push(path_segment);
    }

    // Sort for deterministic output.
    case_names.sort();

    // Write KNOWN_*_CASES constant into generated tests.rs.
    writeln!(dst).unwrap();
    writeln!(dst, "const {const_name}: &[&str] = &[").unwrap();
    for name in &case_names {
        writeln!(dst, "    {:?},", name).unwrap();
    }
    writeln!(dst, "];").unwrap();

    // Write manifest file for staleness detection.
    if let Some(manifest_path) = &suite.manifest_path {
        let new_manifest = case_names.join("\n") + "\n";
        let needs_write = match fs::read_to_string(manifest_path) {
            Ok(existing) => existing != new_manifest,
            Err(_) => true,
        };
        if needs_write {
            fs::write(manifest_path, &new_manifest).expect("failed to write .test_manifest");
        }
        println!("cargo:rerun-if-changed={}", manifest_path.display());
    }

    // Process extra directories.
    for dir in &suite.extra_dirs {
        // Only watch the Cargo.toml, not the entire directory tree.
        println!(
            "cargo:rerun-if-changed={}",
            dir.join("Cargo.toml").display()
        );

        let path_segment = dir.file_name().unwrap().to_str().unwrap().to_owned();

        collect_variants(
            root,
            variants,
            suite.name,
            &suite.expectations_dir,
            &path_segment,
            dir,
            xfail,
        );
    }

    case_names
}

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut dst = File::create(Path::new(&out_dir).join("tests.rs")).unwrap();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tests_dir = manifest_dir.join("tests");

    let mut root = ModNode::new();

    // Parse xfail sets for each suite.
    let xfail_path = tests_dir.join("cbindgen/xfail.txt");
    println!("cargo:rerun-if-changed={}", xfail_path.display());
    let cbindgen_xfail: HashSet<String> = fs::read_to_string(&xfail_path)
        .unwrap_or_default()
        .lines()
        .map(|l| l.trim())
        .filter(|l| !l.is_empty() && !l.starts_with('#'))
        .map(|l| l.to_owned())
        .collect();

    let cbindgen = TestSuite {
        name: "cbindgen",
        cases_dir: tests_dir.join("cbindgen/rust/cases"),
        expectations_dir: tests_dir.join("cbindgen/expectations"),
        extra_dirs: vec![
            tests_dir.join("cbindgen/rust/workspace"),
            tests_dir.join("cbindgen/rust/external_workspace_child"),
        ],
        manifest_path: Some(tests_dir.join("cbindgen/.test_manifest")),
    };

    let cheadergen = TestSuite {
        name: "cheadergen",
        cases_dir: tests_dir.join("cheadergen/rust/cases"),
        expectations_dir: tests_dir.join("cheadergen/expectations"),
        extra_dirs: vec![],
        manifest_path: None,
    };

    let empty_xfail = HashSet::new();

    process_suite(
        &cbindgen,
        &mut dst,
        &mut root,
        VARIANTS,
        "KNOWN_CBINDGEN_CASES",
        &cbindgen_xfail,
    );

    if cheadergen.cases_dir.is_dir() {
        process_suite(
            &cheadergen,
            &mut dst,
            &mut root,
            VARIANTS,
            "KNOWN_CHEADERGEN_CASES",
            &empty_xfail,
        );
    } else {
        writeln!(dst).unwrap();
        writeln!(dst, "const KNOWN_CHEADERGEN_CASES: &[&str] = &[];").unwrap();
    }

    // Emit the nested module tree.
    writeln!(dst).unwrap();
    root.emit(&mut dst, 0);

    dst.flush().unwrap();
}
