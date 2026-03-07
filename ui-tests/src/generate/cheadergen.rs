/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::LazyLock;
use std::{env, fs, str};

use crate::{Language, Style, style_str, tests_dir};

fn compute_metadata(workspace: &Path) -> PathBuf {
    let output = Command::new("cargo")
        .env("LLVM_PROFILE_FILE", "target/profraw/%p_%m.profraw")
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

pub(crate) static CASES_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    get_metadata(
        "CHEADERGEN_CASES_METADATA",
        &tests_dir().join("cbindgen/rust/cases"),
    )
});

pub(crate) static WORKSPACE_METADATA: LazyLock<PathBuf> = LazyLock::new(|| {
    get_metadata(
        "CHEADERGEN_WORKSPACE_METADATA",
        &tests_dir().join("cbindgen/rust/workspace"),
    )
});

pub(crate) fn run_cheadergen(
    path: &Path,
    language: Language,
    cpp_compat: bool,
    style: Option<Style>,
    metadata: &Path,
) -> std::process::Output {
    let cheadergen = env::var("CARGO_BIN_EXE_cheadergen")
        .expect("CARGO_BIN_EXE_cheadergen not set — add cheadergen as a dev-dependency");
    let mut command = Command::new(cheadergen);
    command.env("LLVM_PROFILE_FILE", "target/profraw/%p_%m.profraw");

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
    command
        .output()
        .expect("failed to execute cheadergen — is it built?")
}
