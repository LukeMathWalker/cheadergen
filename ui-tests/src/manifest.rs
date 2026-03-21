/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

use ui_tests_toolkit::{collect_case_dirs, write_manifest_file};

use crate::tests_dir;

pub fn check_manifest_up_to_date(known_cbindgen: &[&str], known_cheadergen: &[&str]) {
    let tests_path = tests_dir();

    // Check cbindgen cases.
    let cbindgen_cases_dir = tests_path.join("cbindgen/rust/cases");
    let cbindgen_manifest_path = tests_path.join("cbindgen/.test_manifest");
    let actual_cbindgen = collect_case_dirs(&cbindgen_cases_dir);

    if actual_cbindgen != known_cbindgen {
        write_manifest_file(&cbindgen_manifest_path, &actual_cbindgen);
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
            let cheadergen_manifest_path = tests_path.join("cheadergen/.test_manifest");
            write_manifest_file(&cheadergen_manifest_path, &actual_cheadergen);
            panic!(
                "cheadergen test manifest is stale — re-run cargo test to pick up new/removed crates.\n\
                 Known: {known_cheadergen:?}\n\
                 Actual: {actual_cheadergen:?}"
            );
        }
    }
}
