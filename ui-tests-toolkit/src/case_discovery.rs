use std::fs;
use std::path::Path;

/// Collect test case directory names under `cases_dir`.
///
/// A directory is considered a test case if it contains a `test.toml` file.
/// Returns sorted directory names.
pub fn collect_case_dirs(cases_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(cases_dir)
        .unwrap_or_else(|e| panic!("failed to read {}: {e}", cases_dir.display()))
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() && entry.path().join("test.toml").exists() {
                Some(entry.file_name().to_str()?.to_owned())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}
