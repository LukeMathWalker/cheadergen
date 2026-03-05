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

    // Watch the cases workspace definition.
    println!(
        "cargo:rerun-if-changed={}",
        cases_dir.join("Cargo.toml").display()
    );

    let mut case_names: Vec<String> = Vec::new();

    for entry in fs::read_dir(&cases_dir).unwrap() {
        let entry = entry.expect("Couldn't read test entry");

        if !entry.file_type().unwrap().is_dir() {
            continue;
        }

        let path_segment = entry.file_name().to_str().unwrap().to_owned();

        // Watch each crate's Cargo.toml for identity changes.
        println!(
            "cargo:rerun-if-changed={}",
            entry.path().join("Cargo.toml").display()
        );

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

        case_names.push(path_segment);
    }

    // Sort for deterministic output.
    case_names.sort();

    // Write KNOWN_CASES constant into generated tests.rs.
    writeln!(dst).unwrap();
    writeln!(dst, "const KNOWN_CASES: &[&str] = &[").unwrap();
    for name in &case_names {
        writeln!(dst, "    {:?},", name).unwrap();
    }
    writeln!(dst, "];").unwrap();

    // Write manifest file for staleness detection.
    let manifest_path = manifest_dir.join("tests").join(".test_manifest");
    let new_manifest = case_names.join("\n") + "\n";
    let needs_write = match fs::read_to_string(&manifest_path) {
        Ok(existing) => existing != new_manifest,
        Err(_) => true,
    };
    if needs_write {
        fs::write(&manifest_path, &new_manifest).expect("failed to write .test_manifest");
    }
    println!("cargo:rerun-if-changed={}", manifest_path.display());

    for dir in &extra_dirs {
        // Only watch the Cargo.toml, not the entire directory tree.
        println!(
            "cargo:rerun-if-changed={}",
            dir.join("Cargo.toml").display()
        );

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
