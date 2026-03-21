/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub(crate) mod cheadergen;

use std::path::Path;
use std::{fs, str};

use crate::{Language, Style, language_extension};
use cheadergen::{
    CBINDGEN_CASES_METADATA, CBINDGEN_WORKSPACE_METADATA, CHEADERGEN_CASES_METADATA,
    run_cheadergen, run_cheadergen_symbols,
};

/// Finds the single file with the expected extension in `output_dir`.
fn find_generated_file(output_dir: &Path, language: Language) -> Vec<u8> {
    let ext = language_extension(language);
    let mut found = Vec::new();
    for entry in fs::read_dir(output_dir).expect("failed to read output dir") {
        let entry = entry.expect("failed to read dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            found.push(path);
        }
    }
    assert!(
        found.len() == 1,
        "expected exactly 1 .{ext} file in {output_dir:?}, found {found:?}"
    );
    fs::read(&found[0]).unwrap_or_else(|e| {
        panic!("failed to read generated file {:?}: {e}", found[0]);
    })
}

const SKIP_WARNING_AS_ERROR_SUFFIX: &str = ".skip_warning_as_error";

/// Invokes cheadergen and returns the raw `Output`.
/// Panics if the binary cannot be spawned (infrastructure failure).
/// Does NOT panic on non-zero exit code.
pub fn invoke_cheadergen(
    _name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
    output_dir: &Path,
) -> std::process::Output {
    let path_str = path.to_str().unwrap();
    let metadata = if path_str.contains("tests/cheadergen/") {
        &*CHEADERGEN_CASES_METADATA
    } else if path_str.contains("/cases/") {
        &*CBINDGEN_CASES_METADATA
    } else {
        &*CBINDGEN_WORKSPACE_METADATA
    };
    run_cheadergen(path, language, cpp_compat, style, output_dir, metadata)
}

pub fn run_generate_test(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");
    let output = invoke_cheadergen(name, path, language, style, cpp_compat, output_dir.path());
    assert!(
        output.status.success(),
        "cheadergen failed for {path:?} with error: {}",
        str::from_utf8(&output.stderr).unwrap_or_default()
    );

    let content = find_generated_file(output_dir.path(), language);
    compare_snapshot(name, path, language, style, cpp_compat, &content);

    // Snapshot stderr diagnostics when the test case opts in via a
    // `snapshot_diagnostics` marker in its `test.toml`.
    if has_snapshot_diagnostics_marker(path) && !output.stderr.is_empty() {
        snapshot_stderr(name, path, language, style, cpp_compat, &output.stderr);
    }
}

pub fn run_symbol_test(name: &str, path: &Path) {
    let path_str = path.to_str().unwrap();
    let metadata = if path_str.contains("tests/cheadergen/") {
        &*CHEADERGEN_CASES_METADATA
    } else if path_str.contains("/cases/") {
        &*CBINDGEN_CASES_METADATA
    } else {
        &*CBINDGEN_WORKSPACE_METADATA
    };

    let symbol_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let symbol_path = symbol_file.path().to_owned();
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    let output = run_cheadergen_symbols(path, &symbol_path, output_dir.path(), metadata);
    assert!(
        output.status.success(),
        "cheadergen --symbol-file failed for {path:?} with error: {}",
        str::from_utf8(&output.stderr).unwrap_or_default()
    );

    let base_name = name
        .strip_suffix(SKIP_WARNING_AS_ERROR_SUFFIX)
        .unwrap_or(name);
    let symbol_content = fs::read_to_string(&symbol_path).expect("failed to read symbol file");
    let snap_name = format!("{base_name}.c.sym");

    let expectations_dir = path.join("expectations");
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(&expectations_dir);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(snap_name, symbol_content);
    });
}

pub fn run_expected_failure_test(
    name: &str,
    variant_path: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");
    let output = invoke_cheadergen(name, path, language, style, cpp_compat, output_dir.path());
    if !output.status.success() {
        // Only snapshot stderr diagnostics for cheadergen tests.
        if path.to_str().unwrap().contains("tests/cheadergen/") {
            snapshot_stderr(name, path, language, style, cpp_compat, &output.stderr);
        }
        return;
    }

    let output_dir_path = output_dir.path().to_owned();
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        let content = find_generated_file(&output_dir_path, language);
        compare_snapshot(name, path, language, style, cpp_compat, &content);
    }));
    if result.is_ok() {
        panic!(
            "xfail test `{name} {variant_path}` now fully passes — \
             remove it from the case's test.toml"
        );
    }
    // Snapshot mismatch: still an expected failure, test passes.
}

pub fn run_expected_symbol_failure_test(name: &str, path: &Path) {
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        run_symbol_test(name, path);
    }));
    if result.is_ok() {
        panic!(
            "xfail symbol test `{name}` now fully passes — \
             remove the symbol xfail marker from the case's test.toml"
        );
    }
}

/// Returns true if the test case directory contains a `snapshot_diagnostics` marker file.
fn has_snapshot_diagnostics_marker(path: &Path) -> bool {
    path.join("snapshot_diagnostics").exists()
}

fn snapshot_stderr(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
    stderr: &[u8],
) {
    let stderr_str = str::from_utf8(stderr).unwrap_or_default();
    if stderr_str.is_empty() {
        return;
    }

    let expectations_dir = path.join("expectations");

    let style_ext = style
        .map(|style| match style {
            Style::Both => "_both",
            Style::Tag => "_tag",
            Style::Type => "",
        })
        .unwrap_or_default();
    let lang_ext = match language {
        Language::Cxx => ".cpp",
        Language::C if cpp_compat => ".compat.c",
        Language::C => ".c",
        Language::Cython => ".pyx",
    };

    let snap_name =
        format!("{name}{style_ext}{lang_ext}.stderr").replace(SKIP_WARNING_AS_ERROR_SUFFIX, "");

    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(&expectations_dir);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(snap_name, stderr_str);
    });
}

#[track_caller]
fn compare_snapshot(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
    content: &[u8],
) {
    let expectations_dir = path.join("expectations");

    let style_ext = style
        .map(|style| match style {
            Style::Both => "_both",
            Style::Tag => "_tag",
            Style::Type => "",
        })
        .unwrap_or_default();
    let lang_ext = match language {
        Language::Cxx => ".cpp",
        Language::C if cpp_compat => ".compat.c",
        Language::C => ".c",
        Language::Cython => ".pyx",
    };

    let source_file =
        format!("{name}{style_ext}{lang_ext}").replace(SKIP_WARNING_AS_ERROR_SUFFIX, "");

    let output = str::from_utf8(content).expect("non-utf8 cheadergen output");

    // Linestyle tests: insta normalizes line endings, so fall back to direct comparison.
    if name.starts_with("linestyle_") {
        let expected_file = expectations_dir.join(&source_file);
        assert!(
            expected_file.exists(),
            "No expectation file found at {expected_file:?}"
        );
        let expected = fs::read_to_string(&expected_file).unwrap();
        assert_eq!(output, expected, "Output mismatch for {source_file}");
        return;
    }

    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(&expectations_dir);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(source_file, output);
    });
}
