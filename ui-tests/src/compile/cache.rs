/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::{Language, Style, tests_dir};

const COMPILE_CACHE_VERSION: &str = "v1";

pub(crate) fn compile_cache_enabled() -> bool {
    env::var("CHEADERGEN_NO_COMPILE_CACHE").is_err()
}

pub(crate) fn compute_compile_hash(
    snap_or_raw: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
    compile_as_cxx: bool,
) -> u64 {
    let mut hasher = std::hash::DefaultHasher::new();
    COMPILE_CACHE_VERSION.hash(&mut hasher);

    // File content
    let content = fs::read(snap_or_raw).unwrap_or_default();
    content.hash(&mut hasher);

    // Compilation parameters
    language.hash(&mut hasher);
    style.hash(&mut hasher);
    skip_warning_as_error.hash(&mut hasher);
    cpp_compat.hash(&mut hasher);
    compile_as_cxx.hash(&mut hasher);

    // Compiler path
    let effective_lang = if compile_as_cxx {
        Language::Cxx
    } else {
        language
    };
    let compiler = match effective_lang {
        Language::Cxx => env::var("CXX").unwrap_or_else(|_| "g++".to_owned()),
        Language::C => env::var("CC").unwrap_or_else(|_| "gcc".to_owned()),
        Language::Cython => env::var("CYTHON").unwrap_or_else(|_| "cython".to_owned()),
    };
    compiler.hash(&mut hasher);

    // Extra flags
    match effective_lang {
        Language::Cxx => {
            if let Ok(flags) = env::var("CXXFLAGS") {
                flags.hash(&mut hasher);
            }
        }
        Language::C => {
            if let Ok(flags) = env::var("CFLAGS") {
                flags.hash(&mut hasher);
            }
        }
        Language::Cython => {}
    }

    // testing-helpers.h content (relevant for C/C++ compiles)
    if matches!(effective_lang, Language::C | Language::Cxx) {
        let helpers_path = tests_dir().join("testing-helpers.h");
        if let Ok(helpers) = fs::read(&helpers_path) {
            helpers.hash(&mut hasher);
        }
    }

    hasher.finish()
}

pub(crate) fn cache_path_for(snap_or_raw: &Path, cpp_compat_cxx: bool) -> PathBuf {
    let mut p = snap_or_raw.as_os_str().to_owned();
    if cpp_compat_cxx {
        p.push(".hash-cxx");
    } else {
        p.push(".hash");
    }
    PathBuf::from(p)
}

pub(crate) fn read_cached_hash(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

pub(crate) fn write_cached_hash(path: &Path, hash: u64) {
    let _ = fs::write(path, hash.to_string());
}

/// Extract the raw content from an insta `.snap` file by stripping the YAML header.
pub(crate) fn read_snap_content(snap_path: &Path) -> String {
    let raw = fs::read_to_string(snap_path).unwrap();
    let rest = raw
        .strip_prefix("---\n")
        .expect("invalid snap file: missing opening ---");
    let idx = rest
        .find("\n---\n")
        .expect("invalid snap file: missing closing ---");
    rest[idx + "\n---\n".len()..].to_string()
}
