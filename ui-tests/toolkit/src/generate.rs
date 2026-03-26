use std::fs;
use std::path::Path;

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

/// Returns true if the test case directory contains a `snapshot_diagnostics` marker file.
pub fn has_snapshot_diagnostics_marker(path: &Path) -> bool {
    path.join("snapshot_diagnostics").exists()
}
