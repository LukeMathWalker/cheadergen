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

fn get_metadata(env_var: &str, workspace: &Path) -> PathBuf {
    if let Ok(path) = env::var(env_var) {
        PathBuf::from(path)
    } else {
        compute_metadata(workspace)
    }
}

static CASES_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    get_metadata(
        "CHEADERGEN_CASES_METADATA",
        Path::new(&crate_dir)
            .join("tests/cbindgen/rust/cases")
            .as_path(),
    )
});

static WORKSPACE_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    get_metadata(
        "CHEADERGEN_WORKSPACE_METADATA",
        Path::new(&crate_dir)
            .join("tests/cbindgen/rust/workspace")
            .as_path(),
    )
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

fn run_generate_test(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let tests_path = Path::new(&crate_dir).join("tests");

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
}

fn run_compile_check(
    expectation: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
) {
    let tmp_dir = tempfile::Builder::new()
        .prefix("cheadergen-test-output")
        .tempdir()
        .expect("Creating tmp dir failed");

    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let tests_path = Path::new(&crate_dir).join("tests");

    compile(
        expectation,
        &tests_path,
        tmp_dir.path(),
        language,
        style,
        skip_warning_as_error,
        cpp_compat,
    );

    if language == Language::C && cpp_compat {
        compile(
            expectation,
            &tests_path,
            tmp_dir.path(),
            Language::Cxx,
            style,
            skip_warning_as_error,
            cpp_compat,
        );
    }
}

macro_rules! generate_variant {
    ($fn_name:ident, $name:expr, $file:tt, $lang:expr, $style:expr, $cpp_compat:expr) => {
        #[test]
        fn $fn_name() {
            run_generate_test($name, Path::new($file), $lang, $style, $cpp_compat);
        }
    };
}

macro_rules! compile_variant {
    ($fn_name:ident, $expectation:expr, $lang:expr, $style:expr, $skip_warn:expr, $cpp_compat:expr) => {
        #[test]
        fn $fn_name() {
            run_compile_check(Path::new($expectation), $lang, $style, $skip_warn, $cpp_compat);
        }
    };
}

// This file is generated by build.rs
include!(concat!(env!("OUT_DIR"), "/tests.rs"));

fn collect_case_dirs(cases_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(cases_dir)
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
    names.sort();
    names
}

#[test]
fn test_manifest_up_to_date() {
    let crate_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let base = Path::new(&crate_dir);

    // Check cbindgen cases.
    let cbindgen_cases_dir = base.join("tests/cbindgen/rust/cases");
    let cbindgen_manifest_path = base.join("tests/cbindgen/.test_manifest");
    let actual_cbindgen = collect_case_dirs(&cbindgen_cases_dir);

    if actual_cbindgen != KNOWN_CBINDGEN_CASES {
        let new_manifest = actual_cbindgen.join("\n") + "\n";
        fs::write(&cbindgen_manifest_path, &new_manifest)
            .expect("failed to write .test_manifest");
        panic!(
            "cbindgen test manifest is stale — re-run cargo test to pick up new/removed crates.\n\
             Known: {KNOWN_CBINDGEN_CASES:?}\n\
             Actual: {actual_cbindgen:?}"
        );
    }

    // Check cheadergen cases (only if directory exists and has entries).
    let cheadergen_cases_dir = base.join("tests/cheadergen/rust/cases");
    if cheadergen_cases_dir.is_dir() {
        let actual_cheadergen = collect_case_dirs(&cheadergen_cases_dir);
        if actual_cheadergen != KNOWN_CHEADERGEN_CASES {
            panic!(
                "cheadergen test manifest is stale — re-run cargo test to pick up new/removed crates.\n\
                 Known: {KNOWN_CHEADERGEN_CASES:?}\n\
                 Actual: {actual_cheadergen:?}"
            );
        }
    }
}
