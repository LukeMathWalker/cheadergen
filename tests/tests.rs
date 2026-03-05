/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::collections::HashSet;
use std::path::Path;
use std::process::Command;
use std::{env, fs, str};

use pretty_assertions::assert_eq;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Language {
    Cxx,
    C,
    Cython,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
enum Style {
    Both,
    Tag,
    Type,
}

fn style_str(style: Style) -> &'static str {
    match style {
        Style::Both => "both",
        Style::Tag => "tag",
        Style::Type => "type",
    }
}

fn run_cbindgen(
    path: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
) -> Vec<u8> {
    let mut command = Command::new("cbindgen");

    match language {
        Language::Cxx => {}
        Language::C => {
            command.arg("--lang").arg("c");
            if cpp_compat {
                command.arg("--cpp-compat");
            }
        }
        Language::Cython => {
            command.arg("--lang").arg("cython");
        }
    }

    if let Some(style) = style {
        command.arg("--style").arg(style_str(style));
    }

    let config = path.with_extension("toml");
    if config.exists() {
        command.arg("--config").arg(config);
    }

    command.arg(path);

    println!("Running: {command:?}");
    let output = command
        .output()
        .expect("failed to execute cbindgen — is it installed and on PATH?");

    assert!(
        output.status.success(),
        "cbindgen failed for {:?} with error: {}",
        path,
        str::from_utf8(&output.stderr).unwrap_or_default()
    );

    output.stdout
}

fn compile(
    cbindgen_output: &Path,
    tests_path: &Path,
    tmp_dir: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
) {
    let cc = match language {
        Language::Cxx => env::var("CXX").unwrap_or_else(|_| "g++".to_owned()),
        Language::C => env::var("CC").unwrap_or_else(|_| "gcc".to_owned()),
        Language::Cython => env::var("CYTHON").unwrap_or_else(|_| "cython".to_owned()),
    };

    let file_name = cbindgen_output
        .file_name()
        .expect("cbindgen output should be a file");
    let mut object = tmp_dir.join(file_name);
    object.set_extension("o");

    let mut command = Command::new(cc);
    match language {
        Language::Cxx | Language::C => {
            command.arg("-D").arg("DEFINED");
            command.arg("-I").arg(tests_path);
            command.arg("-Wall");
            if !skip_warning_as_error {
                command.arg("-Werror");
            }
            command.arg("-Wno-attributes");
            command.arg("-Wno-unused-const-variable");
            command.arg("-Wno-return-type-c-linkage");
            command.arg("-Wno-deprecated-declarations");
            command.arg("-Wno-non-c-typedef-for-linkage");

            if let Language::Cxx = language {
                command.arg("-std=c++17");
                command.arg("-x").arg("c++");
                if let Ok(extra_flags) = env::var("CXXFLAGS") {
                    command.args(extra_flags.split_whitespace());
                }
            } else if let Ok(extra_flags) = env::var("CFLAGS") {
                command.args(extra_flags.split_whitespace());
            }

            if let Some(style) = style {
                command.arg("-D");
                command.arg(format!(
                    "CBINDGEN_STYLE_{}",
                    style_str(style).to_uppercase()
                ));
            }

            if cpp_compat {
                command.arg("-D").arg("CBINDGEN_CPP_COMPAT");
            }

            command.arg("-o").arg(&object);
            command.arg("-c").arg(cbindgen_output);
        }
        Language::Cython => {
            command.arg("-Wextra");
            command.arg("-3");
            command.arg("-o").arg(&object);
            command.arg(cbindgen_output);
        }
    }

    println!("Running: {command:?}");
    let out = command.output().expect("failed to compile");
    assert!(out.status.success(), "Output failed to compile: {out:?}");

    if object.exists() {
        fs::remove_file(object).unwrap();
    }
}

const SKIP_WARNING_AS_ERROR_SUFFIX: &str = ".skip_warning_as_error";

fn run_compile_test(
    name: &'static str,
    path: &Path,
    tmp_dir: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
    cbindgen_outputs: &mut HashSet<Vec<u8>>,
) {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let tests_path = Path::new(&crate_dir).join("tests");
    let expectations_dir = tests_path.join("expectations");

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

    let skip_warning_as_error = name.contains(SKIP_WARNING_AS_ERROR_SUFFIX);

    let source_file =
        format!("{name}{style_ext}{lang_ext}").replace(SKIP_WARNING_AS_ERROR_SUFFIX, "");

    let expected_file = expectations_dir.join(&source_file);

    let bindings_content = run_cbindgen(path, language, cpp_compat, style);

    if cbindgen_outputs.contains(&bindings_content) {
        // Identical output already verified — the expectation file should not exist.
        assert!(
            !expected_file.exists(),
            "Expectation file {expected_file:?} exists but output is identical to a previous run"
        );
    } else {
        if expected_file.exists() {
            let expected = fs::read(&expected_file).unwrap();
            assert_eq!(
                str::from_utf8(&bindings_content).unwrap_or("<non-utf8>"),
                str::from_utf8(&expected).unwrap_or("<non-utf8>"),
                "Output mismatch for {source_file}"
            );
        } else {
            // No expectation file — this is expected when the output is identical
            // to another variant. But if this is the first time seeing this output,
            // that's unexpected.
            panic!("No expectation file found at {expected_file:?} and output is unique");
        }

        cbindgen_outputs.insert(bindings_content);

        // Compile the expected file to verify it's valid C/C++/Cython.
        compile(
            &expected_file,
            &tests_path,
            tmp_dir,
            language,
            style,
            skip_warning_as_error,
            cpp_compat,
        );

        if language == Language::C && cpp_compat {
            compile(
                &expected_file,
                &tests_path,
                tmp_dir,
                Language::Cxx,
                style,
                skip_warning_as_error,
                cpp_compat,
            );
        }
    }
}

fn test_file(name: &'static str, filename: &'static str) {
    let test = Path::new(filename);
    let tmp_dir = tempfile::Builder::new()
        .prefix("cheadergen-test-output")
        .tempdir()
        .expect("Creating tmp dir failed");
    let tmp_dir = tmp_dir.path();

    // Run tests in deduplication priority order. C++ compatibility tests are run first,
    // otherwise we would lose the C++ compiler run if they were deduplicated.
    let mut cbindgen_outputs = HashSet::new();
    for cpp_compat in &[true, false] {
        for style in &[Style::Type, Style::Tag, Style::Both] {
            run_compile_test(
                name,
                test,
                tmp_dir,
                Language::C,
                *cpp_compat,
                Some(*style),
                &mut cbindgen_outputs,
            );
        }
    }

    run_compile_test(
        name,
        test,
        tmp_dir,
        Language::Cxx,
        /* cpp_compat = */ false,
        None,
        &mut HashSet::new(),
    );

    // `Style::Both` should be identical to `Style::Tag` for Cython.
    let mut cbindgen_outputs = HashSet::new();
    for style in &[Style::Type, Style::Tag] {
        run_compile_test(
            name,
            test,
            tmp_dir,
            Language::Cython,
            /* cpp_compat = */ false,
            Some(*style),
            &mut cbindgen_outputs,
        );
    }
}

macro_rules! test_file {
    ($test_function_name:ident, $name:expr, $file:tt) => {
        #[test]
        fn $test_function_name() {
            test_file($name, $file);
        }
    };
}

// This file is generated by build.rs
include!(concat!(env!("OUT_DIR"), "/tests.rs"));
