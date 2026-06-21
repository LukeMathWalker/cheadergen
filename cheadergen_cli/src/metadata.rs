use anyhow::Context as _;
use guppy::MetadataCommand;
use guppy::graph::PackageGraph;
use rustdoc_processor::CrateCollection;
use rustdoc_processor::cache::RustdocGlobalFsCache;
use rustdoc_processor::compute::NoProgress;
use std::path::PathBuf;

use crate::indexing::CheadergenIndexer;

/// The nightly toolchain used for `cargo rustdoc` JSON generation.
/// Must match the FORMAT_VERSION expected by `rustdoc_types`.
/// Single source of truth is `rust-docs-toolchain` at the repo root,
/// also read by the `justfile`.
///
/// Trim trailing ASCII whitespace because `include_str!` preserves the final
/// newline from the file on disk.
pub const DOCS_TOOLCHAIN: &str = trim_ascii_end(include_str!("../rust-docs-toolchain"));

/// Environment variable used to override the default rustdoc toolchain.
pub const DOC_TOOLCHAIN_ENV_VAR: &str = "CHEADERGEN_DOC_TOOLCHAIN";

const fn trim_ascii_end(input: &str) -> &str {
    let bytes = input.as_bytes();
    let mut end = bytes.len();
    while end > 0 {
        match bytes[end - 1] {
            b' ' | b'\n' | b'\r' | b'\t' => end -= 1,
            _ => break,
        }
    }
    input.split_at(end).0
}

/// Load cargo metadata and build a package graph.
///
/// When `metadata_path` is `None`, `cargo metadata` runs in the process's
/// current working directory.
pub fn load_package_graph(metadata_path: Option<&PathBuf>) -> anyhow::Result<PackageGraph> {
    let metadata = if let Some(metadata_path) = metadata_path {
        let json = fs_err::read_to_string(metadata_path)?;
        guppy::CargoMetadata::parse_json(&json)?
    } else {
        MetadataCommand::new().exec()?
    };
    Ok(metadata.build_graph()?)
}

/// On-disk location of the global rustdoc JSON cache.
pub fn cache_dir() -> anyhow::Result<PathBuf> {
    Ok(xdg_home::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get the user's home directory"))?
        .join(".cheadergen/cache"))
}

/// Resolve the toolchain and create a `CrateCollection`.
pub fn create_collection(
    package_graph: PackageGraph,
    toolchain: &str,
) -> anyhow::Result<crate::Collection> {
    let toolchain = toolchain.to_owned();

    let project_fingerprint = package_graph.workspace().root().to_string();

    // Internal-only knob. Workspace-package rustdoc caching isn't yet ready
    // for end-users: the cache key doesn't track every input rustdoc output
    // can depend on (e.g. files pulled in via `include_str!`), so edits in
    // those edge cases would silently yield stale headers. The test suite
    // doesn't exercise those patterns, so it's safe to opt in there.
    let cache_workspace_package_docs =
        std::env::var_os("__CHEADERGEN_CACHE_WORKSPACE_DOCS").is_some();

    let cache_dir = cache_dir()?;
    let disk_cache = RustdocGlobalFsCache::new(
        rustdoc_processor::CRATE_VERSION,
        &toolchain,
        cache_workspace_package_docs,
        &package_graph,
        &cache_dir,
    )?;

    let collection = CrateCollection::new(
        CheadergenIndexer,
        toolchain,
        package_graph,
        project_fingerprint,
        disk_cache,
        Box::new(NoProgress),
    );
    collection
        .bootstrap(std::iter::empty())
        .context("Failed to bootstrap the crate collection")?;

    Ok(collection)
}
