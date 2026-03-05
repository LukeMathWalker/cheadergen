/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::{env, fs, str};

use pretty_assertions::assert_eq;

fn compute_metadata(workspace: &Path) -> PathBuf {
    let output = Command::new("cargo")
        .arg("metadata")
        .arg("--all-features")
        .arg("--format-version")
        .arg("1")
        .arg("--manifest-path")
        .arg(workspace.join("Cargo.toml"))
        .output()
        .expect("failed to run cargo metadata");
    assert!(
        output.status.success(),
        "cargo metadata failed for {workspace:?}: {}",
        str::from_utf8(&output.stderr).unwrap_or_default()
    );
    let metadata_path = workspace.join("metadata.json");
    fs::write(&metadata_path, &output.stdout).expect("failed to write metadata.json");
    metadata_path
}

static CASES_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    compute_metadata(Path::new(&crate_dir).join("tests/rust/cases").as_path())
});

static WORKSPACE_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    compute_metadata(Path::new(&crate_dir).join("tests/rust/workspace").as_path())
});

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
    metadata: &Path,
) -> Vec<u8> {
    let mut command = Command::new("cbindgen");

    command.arg("--metadata").arg(metadata);

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
    name: &str,
    path: &Path,
    tmp_dir: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
    metadata: &Path,
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

    let bindings_content = run_cbindgen(path, language, cpp_compat, style, metadata);

    assert!(
        expected_file.exists(),
        "No expectation file found at {expected_file:?}"
    );

    let expected = fs::read(&expected_file).unwrap();
    assert_eq!(
        str::from_utf8(&bindings_content).unwrap_or("<non-utf8>"),
        str::from_utf8(&expected).unwrap_or("<non-utf8>"),
        "Output mismatch for {source_file}"
    );

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

fn run_test_variant(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("cheadergen-test-output")
        .tempdir()
        .expect("Creating tmp dir failed");

    let filename = path.to_str().unwrap();
    let metadata = if filename.contains("/cases/") {
        &*CASES_METADATA
    } else {
        &*WORKSPACE_METADATA
    };

    run_compile_test(name, path, tmp_dir.path(), language, cpp_compat, style, metadata);
}

macro_rules! test_variant {
    ($fn_name:ident, $name:expr, $file:tt, $lang:expr, $style:expr, $cpp_compat:expr) => {
        #[test]
        fn $fn_name() {
            run_test_variant($name, Path::new($file), $lang, $style, $cpp_compat);
        }
    };
}

// This file is generated by build.rs
include!(concat!(env!("OUT_DIR"), "/tests.rs"));

#[test]
fn test_manifest_up_to_date() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let cases_dir = Path::new(&crate_dir).join("tests/rust/cases");
    let manifest_path = Path::new(&crate_dir).join("tests/.test_manifest");

    let mut actual_cases: Vec<String> = fs::read_dir(&cases_dir)
        .expect("failed to read cases dir")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                Some(entry.file_name().to_str()?.to_owned())
            } else {
                None
            }
        })
        .collect();
    actual_cases.sort();

    if actual_cases != KNOWN_CASES {
        let new_manifest = actual_cases.join("\n") + "\n";
        fs::write(&manifest_path, &new_manifest).expect("failed to write .test_manifest");
        panic!(
            "Test manifest is stale — re-run cargo test to pick up new/removed crates.\n\
             Known: {KNOWN_CASES:?}\n\
             Actual: {actual_cases:?}"
        );
    }
}
