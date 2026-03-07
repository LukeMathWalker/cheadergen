/* This Source Code Form is subject to the terms of the Mozilla Public
 * License, v. 2.0. If a copy of the MPL was not distributed with this
 * file, You can obtain one at http://mozilla.org/MPL/2.0/. */

pub(crate) mod cache;
pub(crate) mod invoke;

use std::fs;
use std::path::Path;

use cache::{
    cache_path_for, compile_cache_enabled, compute_compile_hash, read_cached_hash,
    read_snap_content, write_cached_hash,
};
use invoke::compile;

use crate::{Language, Style, tests_dir};

pub fn run_compile_check(
    snap_or_raw: &Path,
    language: Language,
    style: Option<Style>,
    skip_warning_as_error: bool,
    cpp_compat: bool,
) {
    let use_cache = compile_cache_enabled();

    // Check primary compilation cache.
    let primary_hash = compute_compile_hash(
        snap_or_raw,
        language,
        style,
        skip_warning_as_error,
        cpp_compat,
        false,
    );
    let primary_cache = cache_path_for(snap_or_raw, false);
    let primary_cached =
        use_cache && read_cached_hash(&primary_cache).is_some_and(|h| h == primary_hash);

    // Check secondary (cpp_compat C++ re-compilation) cache.
    let secondary_needed = language == Language::C && cpp_compat;
    let secondary_hash = if secondary_needed {
        compute_compile_hash(
            snap_or_raw,
            language,
            style,
            skip_warning_as_error,
            cpp_compat,
            true,
        )
    } else {
        0
    };
    let secondary_cache = cache_path_for(snap_or_raw, true);
    let secondary_cached = secondary_needed
        && use_cache
        && read_cached_hash(&secondary_cache).is_some_and(|h| h == secondary_hash);

    // If everything is cached, skip entirely.
    if primary_cached && (!secondary_needed || secondary_cached) {
        return;
    }

    let tmp_dir = tempfile::Builder::new()
        .prefix("cheadergen-test-output")
        .tempdir()
        .expect("Creating tmp dir failed");

    let tests_path = tests_dir();

    // For .snap files, extract the content and write to a temp file with the right extension.
    // For raw files (linestyle), use them directly.
    let is_snap = snap_or_raw.extension().is_some_and(|ext| ext == "snap");

    let source_file = if is_snap {
        let content = read_snap_content(snap_or_raw);
        let ext = match language {
            Language::C => "c",
            Language::Cxx => "cpp",
            Language::Cython => "pyx",
        };
        let source_file = tmp_dir.path().join(format!("test.{ext}"));
        fs::write(&source_file, &content).unwrap();
        source_file
    } else {
        snap_or_raw.to_path_buf()
    };

    if !primary_cached {
        compile(
            &source_file,
            &tests_path,
            tmp_dir.path(),
            language,
            style,
            skip_warning_as_error,
            cpp_compat,
        );
        if use_cache {
            write_cached_hash(&primary_cache, primary_hash);
        }
    }

    if secondary_needed && !secondary_cached {
        compile(
            &source_file,
            &tests_path,
            tmp_dir.path(),
            Language::Cxx,
            style,
            skip_warning_as_error,
            cpp_compat,
        );
        if use_cache {
            write_cached_hash(&secondary_cache, secondary_hash);
        }
    }
}
