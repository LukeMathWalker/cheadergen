/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::BTreeMap;
use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use ui_tests_toolkit::{
    VARIANTS, Variant, VariantStatus, collect_case_dirs, read_test_manifest, write_manifest_file,
};

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
    path_segment: &str,
    case_path: &Path,
    emit_generate_without_snap: bool,
) {
    let expectations_dir = case_path.join("expectations");
    let base_name = path_segment
        .strip_suffix(".skip_warning_as_error")
        .unwrap_or(path_segment);

    let identifier_base = path_segment
        .replace(|c: char| !c.is_alphanumeric(), "_")
        .replace("__", "_");

    let skip_warning_as_error = path_segment.contains(".skip_warning_as_error");

    let is_linestyle = path_segment.starts_with("linestyle_");

    let toml_path = case_path.join("test.toml");
    println!("cargo:rerun-if-changed={}", toml_path.display());
    let manifest = read_test_manifest(&toml_path).unwrap_or_else(|e| panic!("{e}"));

    for variant in variants {
        let expectation_file = variant.file_pattern.replace("{name}", base_name);
        let variant_path = variant.module_path.join("/");

        let status = manifest
            .get(&variant_path)
            .copied();

        if status == Some(VariantStatus::Exclude) {
            continue;
        }

        let status_token = match status {
            Some(VariantStatus::Xfail) => "xfail, ",
            Some(VariantStatus::Skip) => "skip, ",
            None => "",
            Some(VariantStatus::Exclude) => unreachable!(),
        };

        // Linestyle tests keep raw files; everything else uses .snap files.
        let resolved_path = if is_linestyle {
            let p = expectations_dir.join(&expectation_file);
            if p.exists() { Some(p) } else { None }
        } else {
            let snap = expectations_dir.join(format!("{expectation_file}.snap"));
            if snap.exists() { Some(snap) } else { None }
        };

        // Emit generate test when a snapshot exists, or unconditionally for
        // non-linestyle suites that opt in (cheadergen). Cbindgen generate
        // tests are only emitted when a snapshot file exists.
        if resolved_path.is_some() || (emit_generate_without_snap && !is_linestyle) {
            let gen_line = format!(
                "generate_variant!({status_token}r#{}, {:?}, {:?}, {:?}, {}, {}, {});",
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
        }

        // Compile test requires the snapshot content, so only emit when it exists.
        if let Some(resolved_path) = resolved_path {
            let compile_line = format!(
                "compile_variant!({status_token}r#{}, {:?}, {:?}, {:?}, {}, {}, {}, {});",
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

    // Emit a symbol test (once per case, not per variant) if a .sym.snap exists.
    let sym_snap = expectations_dir.join(format!("{base_name}.c.sym.snap"));
    if sym_snap.exists() {
        let symbol_status = manifest
            .get("symbol")
            .copied();

        if symbol_status != Some(VariantStatus::Exclude) {
            let symbol_status_token = match symbol_status {
                Some(VariantStatus::Xfail) => "xfail, ",
                Some(VariantStatus::Skip) => "skip, ",
                None => "",
                Some(VariantStatus::Exclude) => unreachable!(),
            };

            let sym_line = format!(
                "symbol_test!({symbol_status_token}r#{}, {:?}, {:?});",
                identifier_base, path_segment, case_path,
            );
            root.insert(&[suite, "symbol"], sym_line);
        }
    }
}

struct TestSuite<'a> {
    name: &'a str,
    cases_dir: PathBuf,
    extra_dirs: Vec<PathBuf>,
    manifest_path: Option<PathBuf>,
    emit_generate_without_snap: bool,
}

fn process_suite(
    suite: &TestSuite,
    dst: &mut File,
    root: &mut ModNode,
    variants: &[Variant],
    const_name: &str,
) -> Vec<String> {
    // Watch the cases workspace definition.
    println!(
        "cargo:rerun-if-changed={}",
        suite.cases_dir.join("Cargo.toml").display()
    );

    let case_names = collect_case_dirs(&suite.cases_dir);

    for path_segment in &case_names {
        let case_path = suite.cases_dir.join(path_segment);

        // Watch each crate's Cargo.toml for identity changes.
        println!(
            "cargo:rerun-if-changed={}",
            case_path.join("Cargo.toml").display()
        );
        // Watch per-case expectations directory.
        println!(
            "cargo:rerun-if-changed={}",
            case_path.join("expectations").display()
        );

        collect_variants(
            root,
            variants,
            suite.name,
            path_segment,
            &case_path,
            suite.emit_generate_without_snap,
        );
    }

    // Write KNOWN_*_CASES constant into generated tests.rs.
    writeln!(dst).unwrap();
    writeln!(dst, "const {const_name}: &[&str] = &[").unwrap();
    for name in &case_names {
        writeln!(dst, "    {:?},", name).unwrap();
    }
    writeln!(dst, "];").unwrap();

    // Write manifest file for staleness detection.
    if let Some(manifest_path) = &suite.manifest_path {
        write_manifest_file(manifest_path, &case_names);
        println!("cargo:rerun-if-changed={}", manifest_path.display());
    }

    // Process extra directories.
    for dir in &suite.extra_dirs {
        // Only watch the Cargo.toml, not the entire directory tree.
        println!(
            "cargo:rerun-if-changed={}",
            dir.join("Cargo.toml").display()
        );
        // Watch per-case expectations directory.
        println!(
            "cargo:rerun-if-changed={}",
            dir.join("expectations").display()
        );

        let path_segment = dir.file_name().unwrap().to_str().unwrap().to_owned();

        collect_variants(
            root,
            variants,
            suite.name,
            &path_segment,
            dir,
            suite.emit_generate_without_snap,
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

    let cbindgen = TestSuite {
        name: "cbindgen",
        cases_dir: tests_dir.join("cbindgen/rust/cases"),
        extra_dirs: vec![
            tests_dir.join("cbindgen/rust/workspace"),
            tests_dir.join("cbindgen/rust/external_workspace_child"),
        ],
        manifest_path: Some(tests_dir.join("cbindgen/.test_manifest")),
        emit_generate_without_snap: false,
    };

    let cheadergen = TestSuite {
        name: "cheadergen",
        cases_dir: tests_dir.join("cheadergen/rust/cases"),
        extra_dirs: vec![],
        manifest_path: Some(tests_dir.join("cheadergen/.test_manifest")),
        emit_generate_without_snap: true,
    };

    process_suite(
        &cbindgen,
        &mut dst,
        &mut root,
        VARIANTS,
        "KNOWN_CBINDGEN_CASES",
    );

    if cheadergen.cases_dir.is_dir() {
        process_suite(
            &cheadergen,
            &mut dst,
            &mut root,
            VARIANTS,
            "KNOWN_CHEADERGEN_CASES",
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
