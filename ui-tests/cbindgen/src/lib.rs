use std::env;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fs, str};

pub use ui_tests_toolkit::types::{Language, Style};
pub use ui_tests_toolkit::{compile, style_str};
use ui_tests_toolkit::cheadergen::{get_metadata, run_cheadergen, run_cheadergen_symbols};
use ui_tests_toolkit::generate::find_generated_file;

const SKIP_WARNING_AS_ERROR_SUFFIX: &str = ".skip_warning_as_error";

fn workspace_root() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest_dir)
        .parent()
        .unwrap()
        .parent()
        .unwrap()
        .to_owned()
}

fn tests_dir() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest_dir).join("tests")
}

static CBINDGEN_CASES_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    get_metadata(
        "CBINDGEN_CASES_METADATA",
        &tests_dir().join("cbindgen/rust/cases"),
    )
});

static CBINDGEN_WORKSPACE_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    get_metadata(
        "CBINDGEN_WORKSPACE_METADATA",
        &tests_dir().join("cbindgen/rust/workspace"),
    )
});

fn testing_helpers_dir() -> PathBuf {
    let toolkit_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../toolkit");
    toolkit_dir.join("data")
}

pub fn run_compile_check(
    expectation: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
) {
    compile::run_compile_check(
        expectation,
        language,
        style,
        skip_warning_as_error,
        cpp_compat,
        &testing_helpers_dir(),
    );
}

pub fn run_compilation_fails_check(
    expectation: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
) {
    compile::run_compilation_fails_check(
        expectation,
        language,
        style,
        skip_warning_as_error,
        cpp_compat,
        &testing_helpers_dir(),
    );
}

fn invoke_cheadergen(
    _name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
    output_dir: &Path,
) -> std::process::Output {
    let path_str = path.to_str().unwrap();
    let metadata = if path_str.contains("/cases/") {
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
    compare_expectation(name, path, language, style, cpp_compat, &content);
}

pub fn run_symbol_test(name: &str, path: &Path) {
    let path_str = path.to_str().unwrap();
    let metadata = if path_str.contains("/cases/") {
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

    let expectations_dir = path.join("expectations");
    let expected_path = expectations_dir.join(format!("{base_name}.c.sym"));
    let expected = fs::read_to_string(&expected_path).unwrap_or_else(|e| {
        panic!("failed to read expectation file {expected_path:?}: {e}");
    });

    assert_equal(&expected, &symbol_content, &format!("{base_name}.c.sym"));
}

/// Test for `header_diff` status: cheadergen succeeds but output differs from the
/// cbindgen expectation. The differing output is captured as an insta snapshot.
pub fn run_header_diff_test(
    name: &str,
    variant_path: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");
    let output = invoke_cheadergen(name, path, language, style, cpp_compat, output_dir.path());
    assert!(
        output.status.success(),
        "header_diff test `{name} {variant_path}` expected cheadergen to succeed, \
         but it failed with: {}",
        str::from_utf8(&output.stderr).unwrap_or_default()
    );

    let content = find_generated_file(output_dir.path(), language);
    let actual = str::from_utf8(&content).expect("non-utf8 cheadergen output");

    // Check if the output now matches the cbindgen expectation (test graduated).
    let expectations_dir = path.join("expectations");
    let cbindgen_file = expectation_filename(name, style, language, cpp_compat);
    let cbindgen_path = expectations_dir.join(&cbindgen_file);
    if fs::read_to_string(&cbindgen_path).is_ok_and(|expected| expected == actual) {
        panic!(
            "header_diff test `{name} {variant_path}` now matches the cbindgen expectation — \
             remove the header_diff marker from the case's test.toml"
        );
    }

    // Snapshot the differing output.
    let diff_snap_name =
        format!("{}.diff", expectation_filename(name, style, language, cpp_compat));

    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(&expectations_dir);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(diff_snap_name, actual);
    });
}

/// Test for `generation_fails` status: cheadergen returns non-zero.
/// Stderr is captured as an insta snapshot.
pub fn run_generation_fails_test(
    name: &str,
    variant_path: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");
    let output = invoke_cheadergen(name, path, language, style, cpp_compat, output_dir.path());
    if output.status.success() {
        panic!(
            "generation_fails test `{name} {variant_path}` expected cheadergen to fail, \
             but it succeeded — remove the generation_fails marker from the case's test.toml"
        );
    }

    let stderr_raw = str::from_utf8(&output.stderr).unwrap_or_default();
    let stderr_str = ui_tests_toolkit::normalize_stderr(stderr_raw, &workspace_root());
    if !stderr_str.is_empty() {
        let expectations_dir = path.join("expectations");
        let snap_name = format!(
            "{}.stderr",
            expectation_filename(name, style, language, cpp_compat)
        );

        let mut settings = insta::Settings::clone_current();
        settings.set_snapshot_path(&expectations_dir);
        settings.set_prepend_module_to_snapshot(false);
        settings.bind(|| {
            insta::assert_snapshot!(snap_name, stderr_str);
        });
    }
}

/// Test for `header_diff` symbol status: cheadergen succeeds but symbol output differs.
pub fn run_header_diff_symbol_test(name: &str, path: &Path) {
    let path_str = path.to_str().unwrap();
    let metadata = if path_str.contains("/cases/") {
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
        "header_diff symbol test expected cheadergen to succeed, but it failed: {}",
        str::from_utf8(&output.stderr).unwrap_or_default()
    );

    let base_name = name
        .strip_suffix(SKIP_WARNING_AS_ERROR_SUFFIX)
        .unwrap_or(name);
    let symbol_content = fs::read_to_string(&symbol_path).expect("failed to read symbol file");

    // Check if it now matches the cbindgen expectation.
    let expectations_dir = path.join("expectations");
    let cbindgen_path = expectations_dir.join(format!("{base_name}.c.sym"));
    if fs::read_to_string(&cbindgen_path).is_ok_and(|expected| expected == symbol_content) {
        panic!(
            "header_diff symbol test `{name}` now matches the cbindgen expectation — \
             remove the header_diff marker from the case's test.toml"
        );
    }

    let diff_snap_name = format!("{base_name}.diff.c.sym");
    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(&expectations_dir);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(diff_snap_name, symbol_content);
    });
}

/// Test for `generation_fails` symbol status: cheadergen fails to generate symbols.
pub fn run_generation_fails_symbol_test(name: &str, path: &Path) {
    let path_str = path.to_str().unwrap();
    let metadata = if path_str.contains("/cases/") {
        &*CBINDGEN_CASES_METADATA
    } else {
        &*CBINDGEN_WORKSPACE_METADATA
    };

    let symbol_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let symbol_path = symbol_file.path().to_owned();
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    let output = run_cheadergen_symbols(path, &symbol_path, output_dir.path(), metadata);
    if output.status.success() {
        panic!(
            "generation_fails symbol test `{name}` expected cheadergen to fail, \
             but it succeeded — remove the generation_fails marker"
        );
    }
}

pub fn check_manifest_up_to_date(known_cbindgen: &[&str]) {
    let tests_path = tests_dir();

    let cbindgen_cases_dir = tests_path.join("cbindgen/rust/cases");
    let cbindgen_manifest_path = tests_path.join("cbindgen/.test_manifest");
    let actual_cbindgen = ui_tests_toolkit::collect_case_dirs(&cbindgen_cases_dir);

    if actual_cbindgen != known_cbindgen {
        ui_tests_toolkit::write_manifest_file(&cbindgen_manifest_path, &actual_cbindgen);
        panic!(
            "cbindgen test manifest is stale — re-run cargo test to pick up new/removed crates.\n\
             Known: {known_cbindgen:?}\n\
             Actual: {actual_cbindgen:?}"
        );
    }
}

fn expectation_filename(
    name: &str,
    style: Option<Style>,
    language: Language,
    cpp_compat: bool,
) -> String {
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
    format!("{name}{style_ext}{lang_ext}").replace(SKIP_WARNING_AS_ERROR_SUFFIX, "")
}

#[track_caller]
fn compare_expectation(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
    content: &[u8],
) {
    let expectations_dir = path.join("expectations");
    let filename = expectation_filename(name, style, language, cpp_compat);
    let output = str::from_utf8(content).expect("non-utf8 cheadergen output");

    let expected_path = expectations_dir.join(&filename);
    let expected = fs::read_to_string(&expected_path).unwrap_or_else(|e| {
        panic!("failed to read expectation file {expected_path:?}: {e}");
    });

    assert_equal(&expected, output, &filename);
}

/// Compare two strings, showing a unified diff on mismatch.
#[track_caller]
fn assert_equal(expected: &str, actual: &str, context: &str) {
    if expected == actual {
        return;
    }

    let diff = similar::TextDiff::from_lines(expected, actual);
    let mut output = format!("Output mismatch for {context}:\n\n");
    for change in diff.iter_all_changes() {
        let sign = match change.tag() {
            similar::ChangeTag::Delete => "-",
            similar::ChangeTag::Insert => "+",
            similar::ChangeTag::Equal => " ",
        };
        output.push_str(sign);
        output.push_str(change.value());
    }
    panic!("{output}");
}
