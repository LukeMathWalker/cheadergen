use std::env;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;
use std::{fs, str};

pub use ui_tests_toolkit::types::{Language, Style};
pub use ui_tests_toolkit::{compile, style_str};
use ui_tests_toolkit::cheadergen::{get_metadata, run_cheadergen, run_cheadergen_symbols};
use ui_tests_toolkit::generate::{find_generated_file, has_snapshot_diagnostics_marker};

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

static CHEADERGEN_CASES_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    get_metadata(
        "CHEADERGEN_CASES_METADATA",
        &tests_dir().join("cheadergen/rust/cases"),
    )
});

fn testing_helpers_dir() -> PathBuf {
    let toolkit_dir = Path::new(env!("CARGO_MANIFEST_DIR")).join("../toolkit");
    toolkit_dir.join("data")
}

pub fn run_compile_check(
    snap_or_raw: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
) {
    compile::run_compile_check(
        snap_or_raw,
        language,
        style,
        skip_warning_as_error,
        cpp_compat,
        &testing_helpers_dir(),
    );
}

pub fn run_compilation_fails_check(
    snap_or_raw: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
) {
    compile::run_compilation_fails_check(
        snap_or_raw,
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
    run_cheadergen(
        path,
        language,
        cpp_compat,
        style,
        output_dir,
        &CHEADERGEN_CASES_METADATA,
    )
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
    let symbol_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let symbol_path = symbol_file.path().to_owned();
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    let output =
        run_cheadergen_symbols(path, &symbol_path, output_dir.path(), &CHEADERGEN_CASES_METADATA);
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

    snapshot_stderr(name, path, language, style, cpp_compat, &output.stderr);
}

/// Test for `generation_fails` symbol status.
pub fn run_generation_fails_symbol_test(name: &str, path: &Path) {
    let symbol_file = tempfile::NamedTempFile::new().expect("failed to create temp file");
    let symbol_path = symbol_file.path().to_owned();
    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    let output =
        run_cheadergen_symbols(path, &symbol_path, output_dir.path(), &CHEADERGEN_CASES_METADATA);
    if output.status.success() {
        panic!(
            "generation_fails symbol test `{name}` expected cheadergen to fail, \
             but it succeeded — remove the generation_fails marker"
        );
    }
}

pub fn check_manifest_up_to_date(known_cheadergen: &[&str]) {
    let tests_path = tests_dir();

    let cheadergen_cases_dir = tests_path.join("cheadergen/rust/cases");
    if cheadergen_cases_dir.is_dir() {
        let actual_cheadergen =
            ui_tests_toolkit::collect_case_dirs(&cheadergen_cases_dir);
        if actual_cheadergen != known_cheadergen {
            let cheadergen_manifest_path = tests_path.join("cheadergen/.test_manifest");
            ui_tests_toolkit::write_manifest_file(&cheadergen_manifest_path, &actual_cheadergen);
            panic!(
                "cheadergen test manifest is stale — re-run cargo test to pick up new/removed crates.\n\
                 Known: {known_cheadergen:?}\n\
                 Actual: {actual_cheadergen:?}"
            );
        }
    }
}

fn snapshot_stderr(
    name: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
    stderr: &[u8],
) {
    let stderr_raw = str::from_utf8(stderr).unwrap_or_default();
    let stderr_str = ui_tests_toolkit::normalize_stderr(stderr_raw, &workspace_root());
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

    let mut settings = insta::Settings::clone_current();
    settings.set_snapshot_path(&expectations_dir);
    settings.set_prepend_module_to_snapshot(false);
    settings.bind(|| {
        insta::assert_snapshot!(source_file, output);
    });
}
