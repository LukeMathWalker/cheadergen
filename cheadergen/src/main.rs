mod codegen;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgAction, Parser, ValueEnum};
use guppy::MetadataCommand;
use guppy::graph::PackageGraph;
use rustdoc_ir::Type;
use rustdoc_processor::CrateCollection;
use rustdoc_processor::cache::RustdocGlobalFsCache;
use rustdoc_processor::compute::NoProgress;
use rustdoc_processor::indexing::NoAnnotations;
use rustdoc_resolver::resolve_free_function;
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
    #[command(subcommand)]
    command: Command,
}

#[derive(Debug, clap::Subcommand)]
enum Command {
    /// Generate C/C++ headers from a Rust crate.
    Generate(GenerateArgs),
    /// Pre-warm the rustdoc JSON cache for all workspace members.
    WarmCache(WarmCacheArgs),
}

#[derive(Debug, Parser)]
struct GenerateArgs {
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

    /// Path to write a symbol file listing exported dynamic symbols.
    #[arg(long)]
    symbol_file: Option<PathBuf>,

    /// Suppress header output. Must be used with --symbol-file.
    #[arg(long)]
    no_header: bool,
}

#[derive(Debug, Parser)]
struct WarmCacheArgs {
    /// Path to the workspace directory (defaults to current directory).
    input: Option<PathBuf>,

    /// Path to a pre-generated `cargo metadata` JSON file.
    #[arg(long)]
    metadata: Option<PathBuf>,

    /// Suppress all output.
    #[arg(short, long)]
    quiet: bool,
}

fn main() -> ExitCode {
    let cli = Cli::parse();

    match cli.command {
        Command::Generate(args) => {
            if args.no_header && args.symbol_file.is_none() {
                eprintln!("Error: --no-header requires --symbol-file");
                return ExitCode::FAILURE;
            }

            if let Err(e) = generate(&args) {
                eprintln!("Error: {e:?}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
        Command::WarmCache(args) => {
            if let Err(e) = warm_cache(&args) {
                eprintln!("Error: {e:?}");
                ExitCode::FAILURE
            } else {
                ExitCode::SUCCESS
            }
        }
    }
}

/// Load cargo metadata and build a package graph.
fn load_package_graph(
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
fn create_collection(
    package_graph: PackageGraph,
) -> anyhow::Result<CrateCollection<NoAnnotations>> {
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

    Ok(collection)
}

fn warm_cache(args: &WarmCacheArgs) -> anyhow::Result<()> {
    let package_graph = load_package_graph(args.metadata.as_ref(), args.input.as_ref())?;

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

    let collection = create_collection(package_graph)?;
    collection
        .compute_batch(workspace_member_ids.into_iter())
        .map_err(|e| anyhow::anyhow!(e))?;

    if !args.quiet {
        eprintln!("Cache warm-up complete.");
    }

    Ok(())
}

fn generate(cli: &GenerateArgs) -> anyhow::Result<()> {
    let package_graph = load_package_graph(cli.metadata.as_ref(), cli.input.as_ref())?;

    // Resolve package info before moving `package_graph` into `CrateCollection`.
    let (package_id, package_name) = {
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
        (package_id, package_name)
    };

    let toolchain =
        std::env::var("CHEADERGEN_DOCS_TOOLCHAIN").unwrap_or_else(|_| DOCS_TOOLCHAIN.to_string());

    if !cli.quiet {
        eprintln!("Computing rustdoc JSON for `{package_name}` using toolchain `{toolchain}`...");
    }

    let collection = create_collection(package_graph)?;

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
        eprintln!("Found {} exported static(s):", exported_static_ids.len());
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

    // Write symbol file if requested.
    if let Some(ref symbol_file) = cli.symbol_file {
        let mut symbols: Vec<String> = Vec::new();
        for id in &extern_c_fn_ids {
            if let Some(name) = krate
                .core
                .krate
                .index
                .get(id)
                .and_then(|item| item.name.clone())
            {
                symbols.push(name);
            }
        }
        for id in &exported_static_ids {
            if let Some(name) = krate
                .core
                .krate
                .index
                .get(id)
                .and_then(|item| item.name.clone())
            {
                symbols.push(name);
            }
        }
        symbols.sort();

        let mut out = String::from("{\n");
        for sym in &symbols {
            out.push_str(sym);
            out.push_str(";\n");
        }
        out.push_str("};\n");
        fs_err::write(symbol_file, &out)?;
    }

    // Resolve each extern "C" function into the IR and generate the header.
    if !cli.no_header {
        let mut resolved_fns = Vec::new();
        for id in &extern_c_fn_ids {
            let item = krate
                .core
                .krate
                .index
                .get(id)
                .ok_or_else(|| anyhow::anyhow!("Missing item for id {:?}", id))?;
            let free_fn = resolve_free_function(&item, krate, &collection)
                .map_err(|e| anyhow::anyhow!("Failed to resolve function: {e}"))?;

            // Bail if any input or output type contains a PathType — we don't handle those yet.
            for input in &free_fn.header.inputs {
                reject_path_types(&input.type_, &free_fn.path.function_name, &input.name)?;
            }
            if let Some(output) = &free_fn.header.output {
                reject_path_types(output, &free_fn.path.function_name, &"return type")?;
            }

            resolved_fns.push(free_fn);
        }

        if !cli.quiet {
            eprintln!("Resolved {} function(s) to IR", resolved_fns.len());
        }

        let mut header = String::new();
        codegen::generate_c_header(&mut resolved_fns, &mut header);

        if let Some(ref output_path) = cli.output {
            fs_err::write(output_path, &header)?;
        } else {
            print!("{header}");
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

/// Bail if `ty` contains a [`rustdoc_ir::PathType`] anywhere — we don't handle named/user-defined
/// types yet.
fn reject_path_types(
    ty: &Type,
    fn_name: &str,
    context: &dyn std::fmt::Display,
) -> anyhow::Result<()> {
    match ty {
        Type::Path(p) => {
            anyhow::bail!(
                "`{fn_name}`: {context} uses named type `{}`, which is not yet supported",
                p.base_type.join("::")
            );
        }
        Type::Reference(r) => reject_path_types(&r.inner, fn_name, context),
        Type::RawPointer(r) => reject_path_types(&r.inner, fn_name, context),
        Type::Tuple(t) => {
            for element in &t.elements {
                reject_path_types(element, fn_name, context)?;
            }
            Ok(())
        }
        Type::Slice(s) => reject_path_types(&s.element_type, fn_name, context),
        Type::Array(a) => reject_path_types(&a.element_type, fn_name, context),
        Type::ScalarPrimitive(_) | Type::Generic(_) => Ok(()),
    }
}
