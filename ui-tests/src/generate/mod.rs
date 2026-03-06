/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub(crate) mod cbindgen;

use std::path::Path;
use std::{fs, str};

use crate::{Language, Style, tests_dir};
use cbindgen::{CASES_METADATA, WORKSPACE_METADATA, run_cbindgen};

const SKIP_WARNING_AS_ERROR_SUFFIX: &str = ".skip_warning_as_error";

pub fn run_generate_test(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    let tests_path = tests_dir();

    let path_str = path.to_str().unwrap();
    let expectations_dir = if path_str.contains("/cbindgen/") {
        tests_path.join("cbindgen/expectations")
    } else {
        tests_path.join("cheadergen/expectations")
    };

    let metadata = if path_str.contains("/cases/") {
        &*CASES_METADATA
    } else {
        &*WORKSPACE_METADATA
    };

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

    let bindings_content = run_cbindgen(path, language, cpp_compat, style, metadata);
    let output = str::from_utf8(&bindings_content).expect("non-utf8 cbindgen output");

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
