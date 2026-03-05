/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

fn main() {
    use std::env;
    use std::fs::{self, File};
    use std::io::Write;
    use std::path::{Path, PathBuf};

    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut dst = File::create(Path::new(&out_dir).join("tests.rs")).unwrap();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let cases_dir = manifest_dir.join("tests").join("rust").join("cases");
    let extra_dirs = [
        manifest_dir.join("tests").join("rust").join("workspace"),
        manifest_dir
            .join("tests")
            .join("rust")
            .join("external_workspace_child"),
    ];

    println!("cargo:rerun-if-changed={}", cases_dir.display());

    for entry in fs::read_dir(&cases_dir).unwrap() {
        let entry = entry.expect("Couldn't read test entry");

        if !entry.file_type().unwrap().is_dir() {
            continue;
        }

        let path_segment = entry.file_name().to_str().unwrap().to_owned();

        let identifier = path_segment
            .replace(|c: char| !c.is_alphanumeric(), "_")
            .replace("__", "_");

        writeln!(
            dst,
            "test_file!(test_{}, {:?}, {:?});",
            identifier,
            path_segment,
            entry.path(),
        )
        .unwrap();
    }

    for dir in &extra_dirs {
        println!("cargo:rerun-if-changed={}", dir.display());

        let path_segment = dir.file_name().unwrap().to_str().unwrap().to_owned();

        let identifier = path_segment
            .replace(|c: char| !c.is_alphanumeric(), "_")
            .replace("__", "_");

        writeln!(
            dst,
            "test_file!(test_{}, {:?}, {:?});",
            identifier, path_segment, dir,
        )
        .unwrap();
    }

    dst.flush().unwrap();
}
