use guppy::MetadataCommand;
use guppy::graph::PackageGraph;
use rustdoc_processor::CrateCollection;
use rustdoc_processor::cache::RustdocGlobalFsCache;
use rustdoc_processor::compute::NoProgress;
use std::path::PathBuf;

use crate::indexing::CheadergenIndexer;

/// The nightly toolchain used for `cargo rustdoc` JSON generation.
/// Must match the FORMAT_VERSION expected by `rustdoc_types`.
pub const DOCS_TOOLCHAIN: &str = "nightly-2025-12-15";

/// Load cargo metadata and build a package graph.
pub fn load_package_graph(
    metadata_path: Option<&PathBuf>,
    input: Option<&PathBuf>,
) -> anyhow::Result<PackageGraph> {
    let metadata = if let Some(metadata_path) = metadata_path {
        let json = fs_err::read_to_string(metadata_path)?;
        guppy::CargoMetadata::parse_json(&json)?
    } else {
        let mut cmd = MetadataCommand::new();
        if let Some(input) = input {
            cmd.current_dir(input);
        }
        cmd.exec()?
    };
    Ok(metadata.build_graph()?)
}

/// Resolve the toolchain and create a `CrateCollection`.
pub fn create_collection(package_graph: PackageGraph) -> anyhow::Result<crate::Collection> {
    let toolchain =
        std::env::var("CHEADERGEN_DOCS_TOOLCHAIN").unwrap_or_else(|_| DOCS_TOOLCHAIN.to_string());

    let project_fingerprint = package_graph.workspace().root().to_string();

    let cache_dir = xdg_home::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get the user's home directory"))?
        .join(".cheadergen/cache");
    let disk_cache = RustdocGlobalFsCache::new(
        rustdoc_processor::CRATE_VERSION,
        &toolchain,
        true,
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
        .expect("Failed to bootstrap the crate collection");

    Ok(collection)
}
