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
    let expectations_dir = manifest_dir.join("tests").join("expectations");
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

    // Watch the expectations directory so new/removed files trigger regeneration.
    println!("cargo:rerun-if-changed={}", expectations_dir.display());

    // Variant definitions: (suffix, style_expr, lang_expr, cpp_compat, file_pattern)
    // file_pattern uses {name} as placeholder for the base name.
    struct Variant {
        suffix: &'static str,
        lang: &'static str,
        style: &'static str,
        cpp_compat: bool,
        file_pattern: &'static str,
    }

    let variants = [
        Variant {
            suffix: "_c",
            lang: "Language::C",
            style: "Some(Style::Type)",
            cpp_compat: false,
            file_pattern: "{name}.c",
        },
        Variant {
            suffix: "_c_tag",
            lang: "Language::C",
            style: "Some(Style::Tag)",
            cpp_compat: false,
            file_pattern: "{name}_tag.c",
        },
        Variant {
            suffix: "_c_both",
            lang: "Language::C",
            style: "Some(Style::Both)",
            cpp_compat: false,
            file_pattern: "{name}_both.c",
        },
        Variant {
            suffix: "_c_compat",
            lang: "Language::C",
            style: "Some(Style::Type)",
            cpp_compat: true,
            file_pattern: "{name}.compat.c",
        },
        Variant {
            suffix: "_c_tag_compat",
            lang: "Language::C",
            style: "Some(Style::Tag)",
            cpp_compat: true,
            file_pattern: "{name}_tag.compat.c",
        },
        Variant {
            suffix: "_c_both_compat",
            lang: "Language::C",
            style: "Some(Style::Both)",
            cpp_compat: true,
            file_pattern: "{name}_both.compat.c",
        },
        Variant {
            suffix: "_cpp",
            lang: "Language::Cxx",
            style: "None",
            cpp_compat: false,
            file_pattern: "{name}.cpp",
        },
        Variant {
            suffix: "_cython",
            lang: "Language::Cython",
            style: "Some(Style::Type)",
            cpp_compat: false,
            file_pattern: "{name}.pyx",
        },
        Variant {
            suffix: "_cython_tag",
            lang: "Language::Cython",
            style: "Some(Style::Tag)",
            cpp_compat: false,
            file_pattern: "{name}_tag.pyx",
        },
    ];

    let mut case_names: Vec<String> = Vec::new();

    let emit_variants_for_case = |dst: &mut File, path_segment: &str, case_path: &Path| {
        // Strip .skip_warning_as_error suffix for expectation file lookup.
        let base_name = path_segment
            .strip_suffix(".skip_warning_as_error")
            .unwrap_or(path_segment);

        let identifier_base = path_segment
            .replace(|c: char| !c.is_alphanumeric(), "_")
            .replace("__", "_");

        for variant in &variants {
            let expectation_file = variant.file_pattern.replace("{name}", base_name);
            if expectations_dir.join(&expectation_file).exists() {
                writeln!(
                    dst,
                    "test_variant!(test_{}{}, {:?}, {:?}, {}, {}, {});",
                    identifier_base,
                    variant.suffix,
                    path_segment,
                    case_path,
                    variant.lang,
                    variant.style,
                    variant.cpp_compat,
                )
                .unwrap();
            }
        }
    };

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

        emit_variants_for_case(&mut dst, &path_segment, &entry.path());

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

        emit_variants_for_case(&mut dst, &path_segment, dir);
    }

    dst.flush().unwrap();
}
