use std::collections::HashSet;
use std::path::PathBuf;

use clap::Parser;

use crate::diagnostic::{DiagnosticSink, Severity, render_diagnostics};
use crate::metadata;

use super::input::{PackageSelection, filter_library_targets, resolve_input, select_packages};

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

    // Skip packages without a library target — cargo rustdoc --lib would fail
    // on them. Explicit `-p` selections still surface as errors below.
    let ws_root: PathBuf = package_graph.workspace().root().to_path_buf().into();
    let debug = std::env::var("CHEADERGEN_DEBUG").is_ok_and(|v| v == "true" || v == "1");
    let mut diagnostics = DiagnosticSink::new(ws_root, debug);
    let explicit_names: HashSet<String> =
        args.package_selection.packages.iter().cloned().collect();
    let packages =
        filter_library_targets(packages, &package_graph, &explicit_names, &mut diagnostics);

    let had_errors = render_pending_diagnostics(&mut diagnostics, debug);

    if packages.is_empty() {
        if had_errors {
            anyhow::bail!("aborting due to previous error(s)");
        }
        if !args.quiet {
            eprintln!("No library crates to warm.");
        }
        return Ok(());
    }

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

    if had_errors {
        anyhow::bail!("aborting due to previous error(s)");
    }
    Ok(())
}

/// Print any buffered diagnostics. Returns `true` if at least one was an error.
fn render_pending_diagnostics(diagnostics: &mut DiagnosticSink, debug: bool) -> bool {
    if diagnostics.is_empty() {
        return false;
    }
    let has_hidden_causes = diagnostics.has_hidden_causes();
    let all = diagnostics.drain();
    let use_color = std::env::var("NO_COLOR").is_err();
    eprint!("{}", render_diagnostics(&all, use_color));
    if !debug && has_hidden_causes {
        eprintln!("note: rerun with `CHEADERGEN_DEBUG=true` for more details");
    } else {
        eprintln!();
    }
    all.iter().any(|d| d.severity == Severity::Error)
}
