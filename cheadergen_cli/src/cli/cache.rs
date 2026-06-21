use std::collections::HashSet;
use std::path::PathBuf;

use clap::Parser;

use crate::diagnostic::{DiagnosticSink, Severity, render_diagnostics};
use crate::metadata;

use super::input::{PackageSelection, filter_library_targets, select_packages};

#[derive(Debug, Parser)]
pub(super) struct CacheArgs {
    #[command(subcommand)]
    command: CacheCommand,
}

#[derive(Debug, clap::Subcommand)]
enum CacheCommand {
    /// Pre-warm the rustdoc JSON cache for the selected workspace members.
    Warm(WarmArgs),
    /// Remove the rustdoc JSON cache from disk.
    Clear,
    /// Print information about the rustdoc JSON cache.
    #[command(subcommand)]
    Show(ShowCommand),
}

#[derive(Debug, clap::Subcommand)]
enum ShowCommand {
    /// Print the cache directory path to stdout.
    Dir,
}

#[derive(Debug, Parser)]
struct WarmArgs {
    /// Directory containing one or more workspace members to warm.
    /// Repeatable. Each path must live inside the workspace rooted at the
    /// current working directory.
    #[arg(long = "input-dir")]
    input_dirs: Vec<PathBuf>,

    #[command(flatten)]
    package_selection: PackageSelection,

    /// Path to a pre-generated `cargo metadata` JSON file.
    #[arg(long)]
    metadata: Option<PathBuf>,

    /// Rust toolchain used by `rustdoc` for generating JSON documentation.
    ///
    /// Be careful: specifying a custom toolchain may break `cheadergen`!
    #[arg(
        long = "rust-toolchain",
        env = metadata::DOC_TOOLCHAIN_ENV_VAR,
        default_value = metadata::DOCS_TOOLCHAIN
    )]
    rust_toolchain: String,

    /// Suppress all output.
    #[arg(short, long)]
    quiet: bool,
}

/// Entry point for the `cache` subcommand.
pub(super) fn run(args: &CacheArgs) -> anyhow::Result<()> {
    match &args.command {
        CacheCommand::Warm(args) => warm(args),
        CacheCommand::Clear => clear(),
        CacheCommand::Show(ShowCommand::Dir) => show_dir(),
    }
}

/// Pre-compute rustdoc JSON for the selected workspace members so later
/// `generate` runs hit the cache.
fn warm(args: &WarmArgs) -> anyhow::Result<()> {
    let package_graph = metadata::load_package_graph(args.metadata.as_ref())?;

    let packages = select_packages(
        &args.input_dirs,
        &args.package_selection,
        &package_graph.workspace(),
    )?;

    // Skip packages without a library target — cargo rustdoc --lib would fail
    // on them. Explicit `-p` selections still surface as errors below.
    let ws_root: PathBuf = package_graph.workspace().root().to_path_buf().into();
    let debug = std::env::var("CHEADERGEN_DEBUG").is_ok_and(|v| v == "true" || v == "1");
    let mut diagnostics = DiagnosticSink::new(ws_root, debug);
    let explicit_names: HashSet<String> = args.package_selection.packages.iter().cloned().collect();
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

    let collection = metadata::create_collection(package_graph, &args.rust_toolchain)?;
    collection.compute_batch(packages.into_iter().map(|(id, _)| id))?;

    if !args.quiet {
        eprintln!("Cache warm-up complete.");
    }

    if had_errors {
        anyhow::bail!("aborting due to previous error(s)");
    }
    Ok(())
}

/// Print the cache directory path to stdout (useful for CI cache configuration).
fn show_dir() -> anyhow::Result<()> {
    println!("{}", metadata::cache_dir()?.display());
    Ok(())
}

/// Remove the on-disk rustdoc JSON cache. Idempotent.
fn clear() -> anyhow::Result<()> {
    let cache_dir = metadata::cache_dir()?;
    if cache_dir.exists() {
        fs_err::remove_dir_all(&cache_dir)?;
        eprintln!("Removed cache at {}.", cache_dir.display());
    } else {
        eprintln!("No cache found at {}.", cache_dir.display());
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

#[cfg(test)]
mod tests {
    use super::WarmArgs;
    use crate::metadata;
    use clap::Parser;

    #[test]
    fn accepts_rust_toolchain_flag() {
        let args = WarmArgs::parse_from(["warm", "--rust-toolchain", "nightly-custom"]);

        assert_eq!(args.rust_toolchain, "nightly-custom");
    }

    #[test]
    fn defaults_rust_toolchain() {
        let args = WarmArgs::parse_from(["warm"]);

        assert_eq!(args.rust_toolchain, metadata::DOCS_TOOLCHAIN);
    }
}
