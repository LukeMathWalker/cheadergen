use std::path::PathBuf;

use clap::Parser;

use crate::metadata;

use super::input::{PackageSelection, resolve_input, select_packages};

#[derive(Debug, Parser)]
pub(super) struct WarmCacheArgs {
    /// Path to the Rust crate directory or its Cargo.toml (defaults to current directory).
    input: Option<PathBuf>,

    #[command(flatten)]
    package_selection: PackageSelection,

    /// Path to a pre-generated `cargo metadata` JSON file.
    #[arg(long)]
    metadata: Option<PathBuf>,

    /// Suppress all output.
    #[arg(short, long)]
    quiet: bool,
}

/// Entry point for the `warm-cache` subcommand — pre-computes rustdoc JSON
/// for all workspace members so later `generate` runs hit the cache.
pub(super) fn warm_cache(args: &WarmCacheArgs) -> anyhow::Result<()> {
    let resolved_input = args
        .input
        .as_ref()
        .map(|p| resolve_input(p))
        .transpose()?;

    let metadata_dir = resolved_input
        .as_ref()
        .map(|r| r.dir().clone())
        .unwrap_or_else(|| PathBuf::from("."));
    let package_graph =
        metadata::load_package_graph(args.metadata.as_ref(), Some(&metadata_dir))?;

    let packages = select_packages(
        resolved_input.as_ref(),
        &args.package_selection,
        &package_graph.workspace(),
    )?;

    if !args.quiet {
        eprintln!(
            "Warming rustdoc cache for {} workspace member(s)...",
            packages.len()
        );
    }

    let collection = metadata::create_collection(package_graph)?;
    collection.compute_batch(packages.into_iter().map(|(id, _)| id))?;

    if !args.quiet {
        eprintln!("Cache warm-up complete.");
    }

    Ok(())
}
