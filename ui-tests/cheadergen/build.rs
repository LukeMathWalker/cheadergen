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

    let cheadergen = TestSuite {
        name: "cheadergen".to_owned(),
        cases_dir: tests_dir.join("cheadergen/rust/cases"),
        extra_dirs: vec![],
        manifest_path: Some(tests_dir.join("cheadergen/.test_manifest")),
        emit_generate_without_snap: true,
        use_snap_files: true,
    };

    if cheadergen.cases_dir.is_dir() {
        process_suite(&cheadergen, &mut dst, &mut root, "KNOWN_CHEADERGEN_CASES");
    } else {
        writeln!(dst).unwrap();
        writeln!(dst, "const KNOWN_CHEADERGEN_CASES: &[&str] = &[];").unwrap();
    }

    // Emit the nested module tree.
    writeln!(dst).unwrap();
    root.emit(&mut dst, 0);

    dst.flush().unwrap();
}
