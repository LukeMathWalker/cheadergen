use std::fs;
use std::path::{Path, PathBuf};

use crate::types::{Language, language_extension};

/// Finds the single file with the expected extension in `output_dir` and returns its content.
pub fn find_generated_file(output_dir: &Path, language: Language) -> Vec<u8> {
    let ext = language_extension(language);
    let mut found = Vec::new();
    for entry in fs::read_dir(output_dir).expect("failed to read output dir") {
        let entry = entry.expect("failed to read dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            found.push(path);
        }
    }
    assert!(
        found.len() == 1,
        "expected exactly 1 .{ext} file in {output_dir:?}, found {found:?}"
    );
    fs::read(&found[0]).unwrap_or_else(|e| {
        panic!("failed to read generated file {:?}: {e}", found[0]);
    })
}

/// Finds all files with the expected extension in `output_dir`, sorted by filename.
///
/// Returns `(filename, content)` pairs. The filename includes the extension
/// (e.g. `"my_crate.h"`).
pub fn find_generated_files(output_dir: &Path, language: Language) -> Vec<(String, Vec<u8>)> {
    let ext = language_extension(language);
    let mut found: Vec<PathBuf> = Vec::new();
    for entry in fs::read_dir(output_dir).expect("failed to read output dir") {
        let entry = entry.expect("failed to read dir entry");
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) == Some(ext) {
            found.push(path);
        }
    }
    assert!(
        !found.is_empty(),
        "expected at least 1 .{ext} file in {output_dir:?}, found none"
    );
    found.sort();
    found
        .into_iter()
        .map(|path| {
            let filename = path
                .file_name()
                .unwrap()
                .to_str()
                .unwrap()
                .to_owned();
            let content = fs::read(&path).unwrap_or_else(|e| {
                panic!("failed to read generated file {path:?}: {e}");
            });
            (filename, content)
        })
        .collect()
}

/// Returns true if the test case directory contains a `snapshot_diagnostics` marker file.
pub fn has_snapshot_diagnostics_marker(path: &Path) -> bool {
    path.join("snapshot_diagnostics").exists()
}
