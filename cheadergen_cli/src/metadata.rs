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
pub const DOCS_TOOLCHAIN: &str = include_str!("../rust-docs-toolchain");

/// Load cargo metadata and build a package graph.
///
/// The metadata JSON is preprocessed via [`strip_self_dev_dep_edges`]
/// before being handed to guppy — see that function for the rationale.
pub fn load_package_graph(
    metadata_path: Option<&PathBuf>,
    input: Option<&PathBuf>,
) -> anyhow::Result<PackageGraph> {
    let raw_json = if let Some(metadata_path) = metadata_path {
        fs_err::read_to_string(metadata_path)?
    } else {
        // Run the `cargo metadata` command guppy would have invoked and
        // capture stdout, so we can rewrite the JSON before parsing it.
        let mut cmd = MetadataCommand::new();
        if let Some(input) = input {
            cmd.current_dir(input);
        }
        let mut command = cmd.cargo_command();
        let output = command.output().map_err(|e| {
            anyhow::anyhow!("failed to invoke `cargo metadata`: {e}")
        })?;
        if !output.status.success() {
            anyhow::bail!(
                "`cargo metadata` failed: {}",
                String::from_utf8_lossy(&output.stderr)
            );
        }
        String::from_utf8(output.stdout)
            .map_err(|e| anyhow::anyhow!("`cargo metadata` produced non-UTF8 output: {e}"))?
    };

    let patched = strip_self_dev_dep_edges(&raw_json)?;
    let metadata = guppy::CargoMetadata::parse_json(&patched)?;
    Ok(metadata.build_graph()?)
}

/// Workaround for <https://github.com/guppy-rs/guppy/pull/586>.
///
/// When a workspace member declares itself as a path dev-dependency (a
/// common idiom for enabling test-only features on yourself in
/// integration tests), guppy's `Sccs::externals` counts the resulting
/// self-loop as an incoming edge, which makes
/// `PackageSet::query_forward([self]).resolve().links(Forward)` return
/// no links. `rustdoc_processor` relies on that iteration to find a
/// dependency by crate name during type resolution, so any rustdoc
/// type whose path crosses into a dep of such a package fails to
/// resolve.
///
/// The bug is fixed upstream in PR #586 but isn't released yet. Until
/// guppy ships a patched version and we bump the minimum requirement,
/// strip self-edges from `resolve.nodes[*].deps[*]` before parsing —
/// guppy builds the package graph from those entries, and removing
/// the self-loop input avoids the buggy code path entirely. Cheadergen
/// never makes use of self-dep edges, so dropping them is safe.
///
/// TODO: delete this and revert to `MetadataCommand::exec()` once
/// cheadergen requires guppy >= the release containing #586.
fn strip_self_dev_dep_edges(raw: &str) -> anyhow::Result<String> {
    let mut value: serde_json::Value = serde_json::from_str(raw)?;

    if let Some(nodes) = value
        .get_mut("resolve")
        .and_then(|r| r.get_mut("nodes"))
        .and_then(|n| n.as_array_mut())
    {
        for node in nodes {
            let Some(id) = node.get("id").and_then(|v| v.as_str()).map(str::to_owned) else {
                continue;
            };
            if let Some(deps) = node.get_mut("deps").and_then(|v| v.as_array_mut()) {
                deps.retain(|d| d.get("pkg").and_then(|v| v.as_str()) != Some(&id));
            }
        }
    }

    Ok(serde_json::to_string(&value)?)
}

/// On-disk location of the global rustdoc JSON cache.
pub fn cache_dir() -> anyhow::Result<PathBuf> {
    Ok(xdg_home::home_dir()
        .ok_or_else(|| anyhow::anyhow!("Failed to get the user's home directory"))?
        .join(".cheadergen/cache"))
}

/// Resolve the toolchain and create a `CrateCollection`.
pub fn create_collection(package_graph: PackageGraph) -> anyhow::Result<crate::Collection> {
    let toolchain =
        std::env::var("CHEADERGEN_DOCS_TOOLCHAIN").unwrap_or_else(|_| DOCS_TOOLCHAIN.to_string());

    let project_fingerprint = package_graph.workspace().root().to_string();

    let cache_dir = cache_dir()?;
    let disk_cache = RustdocGlobalFsCache::new(
        rustdoc_processor::CRATE_VERSION,
        &toolchain,
        false,
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

#[cfg(test)]
mod tests {
    use super::*;

    /// `cargo metadata` fragment matching the structure that triggers
    /// the upstream guppy bug: package `base` has a path-self dev-dep
    /// on itself, expressed as an entry in `resolve.nodes[0].deps[]`
    /// whose `pkg` matches the node's own `id`.
    const RAW_METADATA: &str = r#"{
      "packages": [
        {
          "name": "base",
          "version": "0.1.0",
          "id": "base 0.1.0 (path+file:///x/base)",
          "dependencies": [],
          "manifest_path": "/x/base/Cargo.toml"
        }
      ],
      "resolve": {
        "nodes": [
          {
            "id": "base 0.1.0 (path+file:///x/base)",
            "deps": [
              {
                "name": "base",
                "pkg": "base 0.1.0 (path+file:///x/base)",
                "dep_kinds": [{"kind": "dev", "target": null}]
              },
              {
                "name": "helper",
                "pkg": "helper 0.1.0 (path+file:///x/helper)",
                "dep_kinds": [{"kind": null, "target": null}]
              }
            ]
          }
        ],
        "root": null
      }
    }"#;

    #[test]
    fn strip_self_dev_dep_edges_removes_self_loop() {
        let patched = strip_self_dev_dep_edges(RAW_METADATA).expect("valid JSON");
        let value: serde_json::Value = serde_json::from_str(&patched).unwrap();
        let deps = value["resolve"]["nodes"][0]["deps"].as_array().unwrap();
        assert_eq!(deps.len(), 1, "self-edge should have been removed: {deps:?}");
        assert_eq!(deps[0]["name"], "helper");
    }

    #[test]
    fn strip_self_dev_dep_edges_preserves_non_self_edges() {
        // Two packages, each depending on the other; neither has a
        // self-edge. Nothing should be removed.
        let raw = r#"{
          "packages": [],
          "resolve": {
            "nodes": [
              {
                "id": "a 0.1.0 (path+file:///x/a)",
                "deps": [
                  {"name": "b", "pkg": "b 0.1.0 (path+file:///x/b)", "dep_kinds": []}
                ]
              },
              {
                "id": "b 0.1.0 (path+file:///x/b)",
                "deps": [
                  {"name": "a", "pkg": "a 0.1.0 (path+file:///x/a)", "dep_kinds": []}
                ]
              }
            ]
          }
        }"#;
        let patched = strip_self_dev_dep_edges(raw).expect("valid JSON");
        let value: serde_json::Value = serde_json::from_str(&patched).unwrap();
        let nodes = value["resolve"]["nodes"].as_array().unwrap();
        assert_eq!(nodes[0]["deps"].as_array().unwrap().len(), 1);
        assert_eq!(nodes[1]["deps"].as_array().unwrap().len(), 1);
    }
}
