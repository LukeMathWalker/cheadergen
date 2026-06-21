use std::env;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};

use ui_tests_toolkit::build_script::{ModNode, TestSuite, process_suite};

fn main() {
    let out_dir = PathBuf::from(env::var("OUT_DIR").unwrap());
    let mut dst = File::create(Path::new(&out_dir).join("tests.rs")).unwrap();

    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").unwrap());
    let tests_dir = manifest_dir.join("tests");

    let mut root = ModNode::new();

    let cbindgen = TestSuite {
        name: "cbindgen".to_owned(),
        cases_dir: tests_dir.join("cbindgen/rust/cases"),
        extra_dirs: vec![
            tests_dir.join("cbindgen/rust/workspace"),
            tests_dir.join("cbindgen/rust/external_workspace_child"),
        ],
        manifest_path: Some(tests_dir.join("cbindgen/.test_manifest")),
        emit_generate_without_snap: false,
        use_snap_files: false,
    };

    process_suite(&cbindgen, &mut dst, &mut root, "KNOWN_CBINDGEN_CASES");

    // Emit the nested module tree.
    writeln!(dst).unwrap();
    root.emit(&mut dst, 0);

    dst.flush().unwrap();
}
