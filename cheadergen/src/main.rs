use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgAction, Parser, ValueEnum};
use guppy::MetadataCommand;
use rustdoc_processor::compute::{NoProgress, compute_crate_docs};

/// The nightly toolchain used for `cargo rustdoc` JSON generation.
/// Must match the FORMAT_VERSION expected by `rustdoc_types`.
pub const DOCS_TOOLCHAIN: &str = "nightly-2025-12-15";

#[derive(Debug, Clone, ValueEnum)]
enum Language {
    #[value(name = "c", alias = "C")]
    C,
    #[value(name = "c++", alias = "C++", alias = "cpp")]
    Cxx,
    #[value(name = "cython", alias = "Cython")]
    Cython,
}

#[derive(Debug, Clone, ValueEnum)]
enum Style {
    #[value(name = "both", alias = "Both")]
    Both,
    #[value(name = "tag", alias = "Tag")]
    Tag,
    #[value(name = "type", alias = "Type")]
    Type,
}

/// Generate C/C++ headers from a Rust crate using rustdoc-json.
#[derive(Debug, Parser)]
#[command(version, about)]
struct Cli {
    /// Path to the Rust crate directory (defaults to current directory).
    input: Option<PathBuf>,

    /// Increase verbosity (can be repeated: -v, -vv, -vvv).
    #[arg(short, action = ArgAction::Count)]
    verbose: u8,

    /// Suppress all output.
    #[arg(short, long)]
    quiet: bool,

    /// Verify that the generated bindings match the existing output file.
    #[arg(long)]
    verify: bool,

    /// Path to a TOML configuration file.
    #[arg(short, long)]
    config: Option<PathBuf>,

    /// Target language for the generated bindings.
    #[arg(short, long)]
    lang: Option<Language>,

    /// Add C++ compatibility features to the generated C header.
    #[arg(long)]
    cpp_compat: bool,

    /// Declaration style for generated types.
    #[arg(short, long)]
    style: Option<Style>,

    /// Output file path (defaults to stdout).
    #[arg(short, long)]
    output: Option<PathBuf>,

    /// Path to a pre-generated `cargo metadata` JSON file.
    #[arg(long)]
    metadata: Option<PathBuf>,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    if let Err(e) = run(&cli) {
        eprintln!("Error: {e:?}");
        return ExitCode::FAILURE;
    }
    // We haven't generated any header (yet)
    ExitCode::FAILURE
}

fn run(cli: &Cli) -> anyhow::Result<()> {
    // Get cargo's metadata — either cached or from a fresh invocation
    let metadata = if let Some(ref metadata_path) = cli.metadata {
        let json = fs_err::read_to_string(metadata_path)?;
        guppy::CargoMetadata::parse_json(&json)?
    } else {
        let mut cmd = MetadataCommand::new();
        if let Some(ref input) = cli.input {
            cmd.current_dir(input);
        }
        cmd.exec()?
    };
    let package_graph = metadata.build_graph()?;

    let toolchain =
        std::env::var("CHEADERGEN_DOCS_TOOLCHAIN").unwrap_or_else(|_| DOCS_TOOLCHAIN.to_string());

    let workspace = package_graph.workspace();

    // Resolve the target package: if `cli.input` points to a directory inside the workspace,
    // find the workspace member whose directory matches. Otherwise, use the sole workspace member
    // (or error if ambiguous).
    let package_id = if let Some(ref input) = cli.input {
        let input = input.canonicalize()?;
        let input = camino::Utf8PathBuf::try_from(input)?;
        workspace
            .member_by_path(&input)
            .map_err(|e| anyhow::anyhow!("Could not find workspace member for {input}: {e}"))?
            .id()
            .clone()
    } else {
        let mut members = workspace.iter();
        let first = members
            .next()
            .ok_or_else(|| anyhow::anyhow!("No workspace members found"))?;
        if members.next().is_some() {
            anyhow::bail!("Multiple workspace members found. Pass a path to select one.");
        }
        first.id().clone()
    };

    let package_name = package_graph.metadata(&package_id)?.name().to_string();

    if !cli.quiet {
        eprintln!("Computing rustdoc JSON for `{package_name}` using toolchain `{toolchain}`...");
    }

    let crate_docs = compute_crate_docs(
        &toolchain,
        &package_graph,
        std::iter::once(package_id.clone()),
        workspace.root().as_std_path(),
        &NoProgress,
    )?;

    let krate = crate_docs
        .get(&package_id)
        .ok_or_else(|| anyhow::anyhow!("No documentation returned for `{package_name}`"))?;

    if !cli.quiet {
        let item_count = krate.index.len();
        let root_name = krate
            .index
            .get(&krate.root)
            .and_then(|item| item.name.as_deref())
            .unwrap_or("<unknown>");
        eprintln!(
            "Successfully loaded rustdoc JSON for `{package_name}`: {item_count} items, root module `{root_name}`"
        );
    }

    Ok(())
}
