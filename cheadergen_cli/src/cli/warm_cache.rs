use std::path::PathBuf;

use clap::Parser;

use crate::metadata;

use super::input::resolve_input;

#[derive(Debug, Parser)]
pub(super) struct WarmCacheArgs {
    /// Path to the Rust crate directory or its Cargo.toml (defaults to current directory).
    input: Option<PathBuf>,

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
    let resolved_input = args.input.as_ref().map(|p| resolve_input(p)).transpose()?;
    let package_graph = metadata::load_package_graph(
        args.metadata.as_ref(),
        resolved_input.as_ref().map(|r| r.dir()),
    )?;

    let workspace_member_ids: Vec<_> = package_graph
        .workspace()
        .iter()
        .map(|pkg| pkg.id().clone())
        .collect();

    if !args.quiet {
        eprintln!(
            "Warming rustdoc cache for {} workspace member(s)...",
            workspace_member_ids.len()
        );
    }

    let collection = metadata::create_collection(package_graph)?;
    collection.compute_batch(workspace_member_ids.into_iter())?;

    if !args.quiet {
        eprintln!("Cache warm-up complete.");
    }

    Ok(())
}
