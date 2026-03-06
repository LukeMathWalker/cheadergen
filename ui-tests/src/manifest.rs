/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use std::fs;
use std::path::Path;

use crate::tests_dir;

fn collect_case_dirs(cases_dir: &Path) -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(cases_dir)
        .expect("failed to read cases dir")
        .filter_map(|entry| {
            let entry = entry.ok()?;
            if entry.file_type().ok()?.is_dir() {
                Some(entry.file_name().to_str()?.to_owned())
            } else {
                None
            }
        })
        .collect();
    names.sort();
    names
}

pub fn check_manifest_up_to_date(known_cbindgen: &[&str], known_cheadergen: &[&str]) {
    let tests_path = tests_dir();

    // Check cbindgen cases.
    let cbindgen_cases_dir = tests_path.join("cbindgen/rust/cases");
    let cbindgen_manifest_path = tests_path.join("cbindgen/.test_manifest");
    let actual_cbindgen = collect_case_dirs(&cbindgen_cases_dir);

    if actual_cbindgen != known_cbindgen {
        let new_manifest = actual_cbindgen.join("\n") + "\n";
        fs::write(&cbindgen_manifest_path, &new_manifest).expect("failed to write .test_manifest");
        panic!(
            "cbindgen test manifest is stale — re-run cargo test to pick up new/removed crates.\n\
             Known: {known_cbindgen:?}\n\
             Actual: {actual_cbindgen:?}"
        );
    }

    // Check cheadergen cases (only if directory exists and has entries).
    let cheadergen_cases_dir = tests_path.join("cheadergen/rust/cases");
    if cheadergen_cases_dir.is_dir() {
        let actual_cheadergen = collect_case_dirs(&cheadergen_cases_dir);
        if actual_cheadergen != known_cheadergen {
            panic!(
                "cheadergen test manifest is stale — re-run cargo test to pick up new/removed crates.\n\
                 Known: {known_cheadergen:?}\n\
                 Actual: {actual_cheadergen:?}"
            );
        }
    }
}
