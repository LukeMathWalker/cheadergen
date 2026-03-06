/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

mod compile;
mod generate;
mod manifest;

use std::collections::HashSet;
use std::env;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

pub use compile::run_compile_check;
pub use generate::{invoke_cheadergen, run_generate_test};
pub use manifest::check_manifest_up_to_date;

pub fn tests_dir() -> PathBuf {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    Path::new(&manifest_dir).join("tests")
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Language {
    Cxx,
    C,
    Cython,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Style {
    Both,
    Tag,
    Type,
}

pub fn style_str(style: Style) -> &'static str {
    match style {
        Style::Both => "both",
        Style::Tag => "tag",
        Style::Type => "type",
    }
}

static CBINDGEN_XFAIL: LazyLock<HashSet<String>> = LazyLock::new(|| {
    let manifest_dir = env::var("CARGO_MANIFEST_DIR").unwrap();
    let path = Path::new(&manifest_dir).join("tests/cbindgen/xfail.txt");
    let content = std::fs::read_to_string(&path)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", path.display()));
    content
        .lines()
        .map(|line| line.split('#').next().unwrap().trim())
        .filter(|line| !line.is_empty())
        .map(|line| line.to_owned())
        .collect()
});

pub fn is_xfail(case_name: &str, variant_path: &str) -> bool {
    CBINDGEN_XFAIL.contains(&format!("{case_name} {variant_path}"))
}

pub fn run_xfail_generate_test(
    case_name: &str,
    variant_path: &str,
    path: &Path,
    language: Language,
    style: Option<Style>,
    cpp_compat: bool,
) {
    if !is_xfail(case_name, variant_path) {
        run_generate_test(case_name, path, language, style, cpp_compat);
        return;
    }

    let output = invoke_cheadergen(case_name, path, language, style, cpp_compat);
    assert!(
        !output.status.success(),
        "xfail test `{case_name} {variant_path}` unexpectedly passed (exit code 0) — \
         remove it from xfail.txt"
    );
}
