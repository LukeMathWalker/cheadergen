use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::{env, fs};

use crate::types::Language;

const COMPILE_CACHE_VERSION: &str = "v1";

pub fn compile_cache_enabled() -> bool {
    env::var("CHEADERGEN_NO_COMPILE_CACHE").is_err()
}

pub fn compute_compile_hash(
    snap_or_raw: &Path,
    language: Language,
    style: Option<crate::types::Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
    compile_as_cxx: bool,
    testing_helpers_dir: &Path,
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
        let helpers_path = testing_helpers_dir.join("testing-helpers.h");
        if let Ok(helpers) = fs::read(&helpers_path) {
            helpers.hash(&mut hasher);
        }
    }

    hasher.finish()
}

pub fn cache_path_for(snap_or_raw: &Path, cpp_compat_cxx: bool) -> PathBuf {
    let mut p = snap_or_raw.as_os_str().to_owned();
    if cpp_compat_cxx {
        p.push(".hash-cxx");
    } else {
        p.push(".hash");
    }
    PathBuf::from(p)
}

pub fn read_cached_hash(path: &Path) -> Option<u64> {
    fs::read_to_string(path).ok()?.trim().parse().ok()
}

pub fn write_cached_hash(path: &Path, hash: u64) {
    let _ = fs::write(path, hash.to_string());
}

/// Extract the raw content from an insta `.snap` file by stripping the YAML header.
pub fn read_snap_content(snap_path: &Path) -> String {
    let raw = fs::read_to_string(snap_path).unwrap();
    let rest = raw
        .strip_prefix("---\n")
        .expect("invalid snap file: missing opening ---");
    let idx = rest
        .find("\n---\n")
        .expect("invalid snap file: missing closing ---");
    rest[idx + "\n---\n".len()..].to_string()
}
