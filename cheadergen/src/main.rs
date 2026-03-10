use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgAction, Parser, ValueEnum};
use guppy::MetadataCommand;
use rustdoc_processor::CrateCollection;
use rustdoc_processor::cache::RustdocGlobalFsCache;
use rustdoc_processor::compute::NoProgress;
use rustdoc_processor::indexing::NoAnnotations;
use rustdoc_types::{Abi, Attribute, ItemEnum};

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

    // Resolve package info before moving `package_graph` into `CrateCollection`.
    let (package_id, package_name, project_fingerprint) = {
        let workspace = package_graph.workspace();

        // Resolve the target package: if `cli.input` points to a directory inside the workspace,
        // find the workspace member whose directory matches. Otherwise, use the sole workspace
        // member (or error if ambiguous).
        let package_id = if let Some(ref input) = cli.input {
            let input = input.canonicalize()?;
            let input = camino::Utf8PathBuf::try_from(input)?;
            let input = pathdiff::diff_utf8_paths(input, workspace.root()).expect("Failed to compute the relative path to target crate, with respect to the workspace root");
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
        let project_fingerprint = workspace.root().to_string();
        (package_id, package_name, project_fingerprint)
    };

    if !cli.quiet {
        eprintln!("Computing rustdoc JSON for `{package_name}` using toolchain `{toolchain}`...");
    }

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
        NoAnnotations,
        toolchain,
        package_graph,
        project_fingerprint,
        disk_cache,
        Box::new(NoProgress),
    );
    collection
        .bootstrap(std::iter::empty())
        .expect("Failed to bootstrap the crate collection");

    let krate = collection
        .get_or_compute(&package_id)
        .map_err(|e| anyhow::anyhow!(e))?;

    if !cli.quiet {
        let root_item = krate.core.krate.index.get(&krate.core.krate.root_item_id);
        let root_name = root_item
            .as_ref()
            .and_then(|item| item.name.as_deref())
            .unwrap_or("<unknown>");
        eprintln!(
            "Successfully loaded rustdoc JSON for `{package_name}`: root module `{root_name}`"
        );
    }

    // Collect all free functions with C ABI and exported statics.
    let mut extern_c_fn_ids = Vec::new();
    let mut exported_static_ids = Vec::new();

    for id in krate.import_index.items.keys() {
        let Some(item) = krate.core.krate.index.get(id) else {
            continue;
        };
        match &item.inner {
            ItemEnum::Function(func) if matches!(func.header.abi, Abi::C { .. }) => {
                extern_c_fn_ids.push(*id);
            }
            ItemEnum::Static(_) if has_export_attr(&item.attrs) => {
                exported_static_ids.push(*id);
            }
            _ => {}
        }
    }

    if !cli.quiet {
        eprintln!("Found {} extern \"C\" function(s):", extern_c_fn_ids.len());
        for id in &extern_c_fn_ids {
            let name = krate
                .core
                .krate
                .index
                .get(id)
                .and_then(|item| item.name.clone())
                .unwrap_or_else(|| "<unnamed>".to_string());
            eprintln!("  - {name}");
        }
        eprintln!(
            "Found {} exported static(s):",
            exported_static_ids.len()
        );
        for id in &exported_static_ids {
            let name = krate
                .core
                .krate
                .index
                .get(id)
                .and_then(|item| item.name.clone())
                .unwrap_or_else(|| "<unnamed>".to_string());
            eprintln!("  - {name}");
        }
    }

    Ok(())
}

/// Returns `true` if the item has `#[no_mangle]` or `#[export_name = "..."]`.
fn has_export_attr(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| matches!(a, Attribute::NoMangle | Attribute::ExportName(_)))
}
