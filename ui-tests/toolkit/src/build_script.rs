use std::collections::BTreeMap;
use std::io::Write;
use std::path::{Path, PathBuf};

use crate::{
    VARIANTS, Variant, VariantStatus, collect_case_dirs, read_test_manifest, write_manifest_file,
};

/// Tree node for building nested module structure in generated test code.
pub struct ModNode {
    children: BTreeMap<String, ModNode>,
    test_lines: Vec<String>,
}

impl Default for ModNode {
    fn default() -> Self {
        Self::new()
    }
}

impl ModNode {
    pub fn new() -> Self {
        ModNode {
            children: BTreeMap::new(),
            test_lines: Vec::new(),
        }
    }

    pub fn insert(&mut self, path: &[&str], line: String) {
        if path.is_empty() {
            self.test_lines.push(line);
        } else {
            self.children
                .entry(path[0].to_owned())
                .or_default()
                .insert(&path[1..], line);
        }
    }

    pub fn emit(&self, dst: &mut impl Write, depth: usize) {
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

/// Configuration for a test suite processed by `process_suite`.
pub struct TestSuite {
    /// Module name for generated tests (e.g. "cbindgen", "cheadergen").
    pub name: String,
    /// Directory containing test case subdirectories.
    pub cases_dir: PathBuf,
    /// Additional test directories beyond `cases_dir` (e.g. workspace tests).
    pub extra_dirs: Vec<PathBuf>,
    /// Path to write the `.test_manifest` file for staleness detection.
    pub manifest_path: Option<PathBuf>,
    /// If true, emit generate tests even when no expectation file exists.
    pub emit_generate_without_snap: bool,
    /// If true, look for `.snap` files for expectations. If false, look for plain files.
    pub use_snap_files: bool,
}

/// Process a single test suite: discover cases, emit test functions, write manifest.
///
/// Returns the sorted list of case names found.
pub fn process_suite(
    suite: &TestSuite,
    dst: &mut impl Write,
    root: &mut ModNode,
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

        collect_variants(
            root,
            VARIANTS,
            &suite.name,
            path_segment,
            &case_path,
            suite.emit_generate_without_snap,
            suite.use_snap_files,
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

        let path_segment = dir.file_name().unwrap().to_str().unwrap().to_owned();

        collect_variants(
            root,
            VARIANTS,
            &suite.name,
            &path_segment,
            dir,
            suite.emit_generate_without_snap,
            suite.use_snap_files,
        );
    }

    case_names
}

fn collect_variants(
    root: &mut ModNode,
    variants: &[Variant],
    suite: &str,
    path_segment: &str,
    case_path: &Path,
    emit_generate_without_snap: bool,
    use_snap_files: bool,
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
    let test_manifest = read_test_manifest(&toml_path).unwrap_or_else(|e| panic!("{e}"));

    for variant in variants {
        let expectation_file = variant.file_pattern.replace("{name}", base_name);
        let variant_path = variant.module_path.join("/");

        let status = test_manifest.variants.get(&variant_path).copied();

        if status == Some(VariantStatus::Exclude) {
            continue;
        }

        let status_token = match status {
            Some(VariantStatus::HeaderDiff) => "header_diff, ",
            Some(VariantStatus::GenerationFails) => "generation_fails, ",
            Some(VariantStatus::Skip) => "skip, ",
            // CompilationFails: generation runs normally, only compilation differs.
            Some(VariantStatus::CompilationFails) | None => "",
            Some(VariantStatus::Exclude) => unreachable!(),
        };

        // Resolve the expectation file path.
        // - Linestyle tests always use raw files (no .snap).
        // - When use_snap_files is true, look for .snap files.
        // - When use_snap_files is false, look for plain files.
        let resolved_path = if is_linestyle || !use_snap_files {
            let p = expectations_dir.join(&expectation_file);
            if p.exists() { Some(p) } else { None }
        } else {
            let snap = expectations_dir.join(format!("{expectation_file}.snap"));
            if snap.exists() { Some(snap) } else { None }
        };

        // Emit generate test when an expectation exists, unconditionally for
        // suites that opt in (cheadergen), or for header_diff/generation_fails
        // tests (which always run, even without a pre-existing expectation).
        let has_status = status.is_some();
        if resolved_path.is_some() || (emit_generate_without_snap && !is_linestyle) || has_status {
            let package_arg = match &test_manifest.package {
                Some(name) => format!("Some({name:?})"),
                None => "None::<&str>".to_string(),
            };
            let gen_line = format!(
                "generate_variant!({status_token}r#{}, {:?}, {:?}, {:?}, {}, {}, {}, {});",
                identifier_base,
                path_segment,
                variant_path,
                case_path,
                variant.lang,
                variant.style,
                variant.cpp_compat,
                package_arg,
            );
            let mut gen_path: Vec<&str> = vec![suite, "generate"];
            gen_path.extend_from_slice(variant.module_path);
            root.insert(&gen_path, gen_line);
        }

        // Compile test requires expectation content.
        // - For normal/skip/compilation_fails tests: use the normal expectation file.
        // - For header_diff: use the .diff.{ext}.snap file (snapshot of cheadergen's output).
        // - For generation_fails: no compile test (no header to compile).
        let compile_path_resolved = match status {
            Some(VariantStatus::GenerationFails | VariantStatus::HeaderDiff) => None,
            _ => resolved_path,
        };

        if let Some(compile_expectation) = compile_path_resolved {
            let compile_status_token = match status {
                Some(VariantStatus::CompilationFails) => "compilation_fails, ",
                Some(VariantStatus::Skip) => "skip, ",
                _ => "",
            };
            let compile_line = format!(
                "compile_variant!({compile_status_token}r#{}, {:?}, {:?}, {:?}, {}, {}, {}, {});",
                identifier_base,
                path_segment,
                variant_path,
                compile_expectation,
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

    // Emit a symbol test (once per case, not per variant).
    // Check for both .sym.snap and plain .c.sym depending on mode.
    let sym_path = if use_snap_files {
        let p = expectations_dir.join(format!("{base_name}.c.sym.snap"));
        if p.exists() { Some(p) } else { None }
    } else {
        let p = expectations_dir.join(format!("{base_name}.c.sym"));
        if p.exists() { Some(p) } else { None }
    };

    if sym_path.is_some() {
        let symbol_status = test_manifest.variants.get("symbol").copied();

        if symbol_status != Some(VariantStatus::Exclude) {
            let symbol_status_token = match symbol_status {
                Some(VariantStatus::HeaderDiff) => "header_diff, ",
                Some(VariantStatus::GenerationFails) => "generation_fails, ",
                Some(VariantStatus::Skip) => "skip, ",
                // CompilationFails is not meaningful for symbol tests (no compilation),
                // so treat it the same as normal.
                Some(VariantStatus::CompilationFails) | None => "",
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
