/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub(crate) mod cheadergen;

use std::path::Path;
use std::{fs, str};

use crate::{Language, Style};
use cheadergen::{
    CBINDGEN_CASES_METADATA, CBINDGEN_WORKSPACE_METADATA, CHEADERGEN_CASES_METADATA, run_cheadergen,
    run_cheadergen_symbols,
};

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
) -> std::process::Output {
    let path_str = path.to_str().unwrap();
    let metadata = if path_str.contains("tests/cheadergen/") {
        &*CHEADERGEN_CASES_METADATA
    } else if path_str.contains("/cases/") {
        &*CBINDGEN_CASES_METADATA
    } else {
        &*CBINDGEN_WORKSPACE_METADATA
    };
    run_cheadergen(path, language, cpp_compat, style, metadata)
}

pub fn run_generate_test(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    let output = invoke_cheadergen(name, path, language, style, cpp_compat);
    assert!(
        output.status.success(),
        "cheadergen failed for {path:?} with error: {}",
        str::from_utf8(&output.stderr).unwrap_or_default()
    );
    compare_snapshot(name, path, language, style, cpp_compat, &output.stdout);
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

    let output = run_cheadergen_symbols(path, &symbol_path, metadata);
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
    let output = invoke_cheadergen(name, path, language, style, cpp_compat);
    if !output.status.success() {
        return;
    }
    let result = std::panic::catch_unwind(std::panic::AssertUnwindSafe(|| {
        compare_snapshot(name, path, language, style, cpp_compat, &output.stdout);
    }));
    if result.is_ok() {
        panic!(
            "xfail test `{name} {variant_path}` now fully passes — \
             remove it from the case's test.toml"
        );
    }
    // Snapshot mismatch: still an expected failure, test passes.
}

fn compare_snapshot(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
    stdout: &[u8],
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

    let output = str::from_utf8(stdout).expect("non-utf8 cheadergen output");

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
