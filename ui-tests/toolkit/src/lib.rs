// Build-time modules
pub(crate) mod case_discovery;
pub(crate) mod test_manifest;
pub(crate) mod variant;
pub(crate) mod variant_status;
pub mod build_script;

// Runtime modules
pub mod cheadergen;
pub mod compile;
pub mod generate;
pub mod types;

// Build-time re-exports
pub use case_discovery::collect_case_dirs;
pub use test_manifest::{TestManifest, TestManifestError, read_manifest_file, read_test_manifest, write_manifest_file};
pub use variant::{VARIANTS, Variant, variant_path_strings};
pub use variant_status::VariantStatus;

// Runtime re-exports
pub use types::{Language, Style, language_extension, style_str};

use std::path::Path;

/// Replace the workspace root in stderr so snapshots are portable across checkouts.
pub fn normalize_stderr(stderr: &str, workspace_root: &Path) -> String {
    let root_str = workspace_root
        .to_str()
        .expect("non-UTF-8 workspace root");
    stderr.replace(root_str, "[WORKSPACE]")
}
