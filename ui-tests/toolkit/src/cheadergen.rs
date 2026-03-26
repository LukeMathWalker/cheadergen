use std::path::{Path, PathBuf};
use std::process::Command;
use std::{env, fs, str};

use crate::types::{Language, Style, style_str};

/// Compute `cargo metadata` for a workspace, writing the result to `metadata.json`
/// in the workspace root. Returns the path to the metadata file.
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

/// Resolve a metadata path from an environment variable, falling back to computing
/// it from `cargo metadata` if the variable is not set.
pub fn get_metadata(env_var: &str, workspace: &Path) -> PathBuf {
    if let Ok(path) = env::var(env_var) {
        PathBuf::from(path)
    } else {
        compute_metadata(workspace)
    }
}

/// Invoke cheadergen to generate symbols, returning the raw `Output`.
pub fn run_cheadergen_symbols(
    path: &Path,
    symbol_file: &Path,
    output_dir: &Path,
    metadata: &Path,
) -> std::process::Output {
    let cheadergen = env::var("CARGO_BIN_EXE_cheadergen")
        .expect("CARGO_BIN_EXE_cheadergen not set — add cheadergen as a dev-dependency");
    let mut command = Command::new(cheadergen);

    command.arg("generate");
    command.arg("--quiet");
    command.env("NO_COLOR", "1");
    command.arg("--metadata").arg(metadata);
    command.arg("--no-header");
    command.arg("--symbol-file").arg(symbol_file);
    command.arg("--lang").arg("c");
    command.arg("--style").arg("type");
    command.arg("--output-dir").arg(output_dir);

    let config = path.join("cheadergen.toml");
    if config.exists() {
        command.arg("--config").arg(config);
    }

    command.arg(path);

    println!("Running: {command:?}");
    command
        .output()
        .expect("failed to execute cheadergen — is it built?")
}

/// Invoke cheadergen to generate a header file, returning the raw `Output`.
pub fn run_cheadergen(
    path: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
    output_dir: &Path,
    metadata: &Path,
) -> std::process::Output {
    let cheadergen = env::var("CARGO_BIN_EXE_cheadergen")
        .expect("CARGO_BIN_EXE_cheadergen not set — add cheadergen as a dev-dependency");
    let mut command = Command::new(cheadergen);

    command.arg("generate");
    command.arg("--quiet");
    command.env("NO_COLOR", "1");
    command.arg("--metadata").arg(metadata);

    match language {
        Language::C => command.arg("--lang").arg("c"),
        Language::Cxx => command.arg("--lang").arg("c++"),
        Language::Cython => command.arg("--lang").arg("cython"),
    };

    if cpp_compat {
        command.arg("--cpp-compat");
    }

    if let Some(style) = style {
        command.arg("--style").arg(style_str(style));
    }

    command.arg("--output-dir").arg(output_dir);

    let config = path.join("cheadergen.toml");
    if config.exists() {
        command.arg("--config").arg(config);
    }

    command.arg(path);

    println!("Running: {command:?}");
    command
        .output()
        .expect("failed to execute cheadergen — is it built?")
}
