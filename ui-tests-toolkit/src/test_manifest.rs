use std::collections::HashMap;
use std::fs;
use std::path::Path;

use crate::variant::variant_path_strings;
use crate::variant_status::VariantStatus;

/// Read and parse a `test.toml` manifest, returning a map of variant path to status.
///
/// Only entries with explicit status overrides are included in the returned map;
/// variants not mentioned in the file are implicitly `Normal`.
///
/// Returns an error if the file cannot be read, parsed, or contains unknown variant keys.
pub fn read_test_manifest(
    toml_path: &Path,
) -> Result<HashMap<String, VariantStatus>, TestManifestError> {
    let content = fs::read_to_string(toml_path).map_err(|e| TestManifestError::Io {
        path: toml_path.display().to_string(),
        source: e,
    })?;
    let manifest: HashMap<String, VariantStatus> =
        toml::from_str(&content).map_err(|e| TestManifestError::Parse {
            path: toml_path.display().to_string(),
            source: e,
        })?;

    let valid = variant_path_strings();
    let mut unknown: Vec<String> = manifest
        .keys()
        .filter(|k| !valid.iter().any(|v| v == k.as_str()))
        .cloned()
        .collect();
    if !unknown.is_empty() {
        unknown.sort();
        return Err(TestManifestError::UnknownKeys {
            path: toml_path.display().to_string(),
            keys: unknown,
        });
    }

    Ok(manifest)
}

/// Read a `.test_manifest` file, returning the list of case names (one per line).
pub fn read_manifest_file(path: &Path) -> Vec<String> {
    match fs::read_to_string(path) {
        Ok(content) => content
            .lines()
            .filter(|l| !l.is_empty())
            .map(|l| l.to_owned())
            .collect(),
        Err(_) => Vec::new(),
    }
}

/// Write a `.test_manifest` file with sorted case names, one per line.
/// Only performs the write if the content has changed.
/// Returns `true` if the file was written.
pub fn write_manifest_file(path: &Path, names: &[String]) -> bool {
    let new_content = names.join("\n") + "\n";
    let needs_write = match fs::read_to_string(path) {
        Ok(existing) => existing != new_content,
        Err(_) => true,
    };
    if needs_write {
        fs::write(path, &new_content).expect("failed to write .test_manifest");
    }
    needs_write
}

#[derive(Debug)]
pub enum TestManifestError {
    Io {
        path: String,
        source: std::io::Error,
    },
    Parse {
        path: String,
        source: toml::de::Error,
    },
    UnknownKeys {
        path: String,
        keys: Vec<String>,
    },
}

impl std::fmt::Display for TestManifestError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            TestManifestError::Io { path, source } => {
                write!(f, "failed to read {path}: {source}")
            }
            TestManifestError::Parse { path, source } => {
                write!(f, "invalid {path}: {source}")
            }
            TestManifestError::UnknownKeys { path, keys } => {
                write!(f, "unknown variant(s) {keys:?} in {path}")
            }
        }
    }
}

impl std::error::Error for TestManifestError {}
