use std::io::Write;
use std::process::Command;

fn run_config_error(config_toml: &str, extra_args: &[&str]) -> String {
    let bin = std::env::var("CHEADERGEN_BIN").expect("CHEADERGEN_BIN must be set by setup script");

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    tmp.write_all(config_toml.as_bytes())
        .expect("failed to write config");

    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new(&bin)
        .arg("generate")
        .arg("--config")
        .arg(tmp.path())
        .arg("--output-dir")
        .arg(output_dir.path())
        .args(extra_args)
        // Ensure consistent output for CLI errors
        .env("RUST_BACKTRACE", "0")
        .env("RUST_LIB_BACKTRACE", "0")
        .output()
        .expect("failed to run cheadergen");

    assert!(
        !output.status.success(),
        "expected non-zero exit code, got {:?}",
        output.status
    );

    String::from_utf8(output.stderr).expect("stderr is not valid UTF-8")
}

#[test]
fn cxx_not_supported() {
    let stderr = run_config_error("", &["--lang", "c++"]);
    insta::assert_snapshot!(stderr, @"Error: C++ output is not yet supported
");
}

#[test]
fn unknown_field_rejected() {
    let stderr = run_config_error("bogus = true", &["--lang", "c"]);
    insta::assert_snapshot!(stderr, @r###"
    Error: failed to parse config file: TOML parse error at line 1, column 1
      |
    1 | bogus = true
      | ^^^^^
    unknown field `bogus`, expected one of `preamble`, `trailer`, `autogen_warning`, `pragma_once`, `includes`, `no_includes`, `after_includes`, `documentation`, `documentation_style`, `documentation_length`, `sort_by`, `fn`, `static`, `constant`, `enum`, `bundle`, `package`, `header`, `c`, `c++`, `cpp`, `cxx`
    "###);
}

#[test]
fn skip_empty_conflicts_with_bundle_cli() {
    // clap rejects `--skip-empty --bundle` before any work is done.
    let stderr = run_config_error("", &["--lang", "c", "--skip-empty", "--bundle"]);
    insta::assert_snapshot!(stderr);
}

#[test]
fn skip_empty_conflicts_with_bundle_in_config() {
    // clap can't see `bundle = true` in the config file — the runtime check
    // catches it instead.
    let stderr = run_config_error("bundle = true", &["--lang", "c", "--skip-empty"]);
    insta::assert_snapshot!(stderr, @"Error: --skip-empty is only valid in partitioned mode; remove --bundle to use it
");
}

#[test]
fn cython_rejected() {
    let bin = std::env::var("CHEADERGEN_BIN").expect("CHEADERGEN_BIN must be set by setup script");

    let output_dir = tempfile::tempdir().expect("failed to create temp dir");

    let output = Command::new(&bin)
        .arg("generate")
        .arg("--lang")
        .arg("cython")
        .arg("--output-dir")
        .arg(output_dir.path())
        // Ensure consistent output for CLI errors
        .env("RUST_BACKTRACE", "0")
        .env("RUST_LIB_BACKTRACE", "0")
        .output()
        .expect("failed to run cheadergen");

    assert!(
        !output.status.success(),
        "expected non-zero exit code, got {:?}",
        output.status
    );

    let stderr = String::from_utf8(output.stderr).expect("stderr is not valid UTF-8");
    insta::assert_snapshot!(stderr, @"Error: Cython output is not yet supported
");
}
