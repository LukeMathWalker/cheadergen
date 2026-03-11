use std::io::Write;
use std::process::Command;

fn run_config_error(config_toml: &str, extra_args: &[&str]) -> String {
    let bin = std::env::var("CHEADERGEN_BIN").expect("CHEADERGEN_BIN must be set by setup script");

    let mut tmp = tempfile::NamedTempFile::new().expect("failed to create temp file");
    tmp.write_all(config_toml.as_bytes())
        .expect("failed to write config");

    let output = Command::new(&bin)
        .arg("generate")
        .arg("--config")
        .arg(tmp.path())
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
fn cpp_compat_rejected_for_cxx() {
    let stderr = run_config_error("", &["--lang", "c++", "--cpp-compat"]);
    insta::assert_snapshot!(stderr, @"Error: `--cpp-compat` is not supported for C++ output (it is only meaningful for C headers)
");
}

#[test]
fn style_rejected_for_cxx() {
    let stderr = run_config_error("", &["--lang", "c++", "--style", "both"]);
    insta::assert_snapshot!(stderr, @"Error: `--style` is not supported for C++ output (C++ does not use typedef-style declarations)
");
}

#[test]
fn unknown_field_rejected() {
    let stderr = run_config_error("bogus = true", &["--lang", "c"]);
    insta::assert_snapshot!(stderr, @r"
    Error: failed to parse config file: TOML parse error at line 1, column 1
      |
    1 | bogus = true
      | ^^^^^
    unknown field `bogus`, expected one of `header`, `trailer`, `autogen_warning`, `include_guard`, `pragma_once`, `sys_includes`, `includes`, `no_includes`, `after_includes`, `c`, `c++`, `cpp`, `cxx`
    ");
}

#[test]
fn cython_rejected() {
    let bin = std::env::var("CHEADERGEN_BIN").expect("CHEADERGEN_BIN must be set by setup script");

    let output = Command::new(&bin)
        .arg("generate")
        .arg("--lang")
        .arg("cython")
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
