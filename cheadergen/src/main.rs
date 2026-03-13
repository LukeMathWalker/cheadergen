mod analysis;
mod codegen;
mod config;
mod constant_item;
mod metadata;
mod static_item;

use std::path::PathBuf;
use std::process::ExitCode;

use clap::{ArgAction, Parser};

use config::{Language, Style};

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
    /// Configuration utilities.
    Config(ConfigArgs),
}

#[derive(Debug, Parser)]
struct ConfigArgs {
    #[command(subcommand)]
    command: ConfigCommand,
}

#[derive(Debug, clap::Subcommand)]
enum ConfigCommand {
    /// Translate a cbindgen config file into cheadergen format.
    Translate(TranslateArgs),
}

#[derive(Debug, Parser)]
struct TranslateArgs {
    /// Path to the cbindgen config file.
    #[arg(long)]
    from: PathBuf,
    /// Path to write the cheadergen config file.
    #[arg(long)]
    to: PathBuf,
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
    lang: Language,

    /// Add C++ compatibility features to the generated C header (C only).
    #[arg(long)]
    cpp_compat: bool,

    /// Declaration style for generated types (C only).
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
        Command::Config(args) => match args.command {
            ConfigCommand::Translate(args) => {
                if let Err(e) = config::cbindgen::translate(&args.from, &args.to) {
                    eprintln!("Error: {e}");
                    ExitCode::FAILURE
                } else {
                    ExitCode::SUCCESS
                }
            }
        },
    }
}

fn warm_cache(args: &WarmCacheArgs) -> anyhow::Result<()> {
    let package_graph =
        metadata::load_package_graph(args.metadata.as_ref(), args.input.as_ref())?;

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
    collection
        .compute_batch(workspace_member_ids.into_iter())
        .map_err(|e| anyhow::anyhow!(e))?;

    if !args.quiet {
        eprintln!("Cache warm-up complete.");
    }

    Ok(())
}

fn generate(cli: &GenerateArgs) -> anyhow::Result<()> {
    // Load config file (or use defaults).
    let raw_config = if let Some(ref config_path) = cli.config {
        config::RawConfig::from_toml_file(config_path)?
    } else {
        config::RawConfig::default()
    };

    // Build CLI overrides.
    let overrides = config::CliOverrides {
        style: cli.style.clone(),
        cpp_compat: cli.cpp_compat,
    };

    // Validate config against the selected language.
    let config = raw_config.into_config(&cli.lang, &overrides)?;

    let package_graph = metadata::load_package_graph(cli.metadata.as_ref(), cli.input.as_ref())?;

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

    let toolchain = std::env::var("CHEADERGEN_DOCS_TOOLCHAIN")
        .unwrap_or_else(|_| metadata::DOCS_TOOLCHAIN.to_string());

    if !cli.quiet {
        eprintln!("Computing rustdoc JSON for `{package_name}` using toolchain `{toolchain}`...");
    }

    let collection = metadata::create_collection(package_graph)?;

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

    let (fn_sort_by, static_sort_by, constant_sort_by) = match &config {
        config::Config::C(c) => (
            c.common.fn_sort_by,
            c.common.static_sort_by,
            c.common.constant_sort_by,
        ),
        config::Config::Cxx(cxx) => (
            cxx.common.fn_sort_by,
            cxx.common.static_sort_by,
            cxx.common.constant_sort_by,
        ),
    };
    let extern_items =
        analysis::find_extern_items(krate, fn_sort_by, static_sort_by, constant_sort_by);

    if !cli.quiet {
        eprintln!(
            "Found {} extern \"C\" function(s):",
            extern_items.fn_ids.len()
        );
        for id in &extern_items.fn_ids {
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
            extern_items.static_ids.len()
        );
        for id in &extern_items.static_ids {
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
        let symbols = analysis::collect_symbols(&extern_items, krate);
        codegen::write_symbol_file(&symbols, symbol_file)?;
    }

    // Resolve each extern "C" function and static into the IR and generate the header.
    if !cli.no_header {
        let resolved_fns =
            analysis::resolve_functions(&extern_items.fn_ids, krate, &collection)?;
        let resolved_statics =
            analysis::resolve_statics(&extern_items.static_ids, krate, &collection)?;
        let resolved_constants =
            analysis::resolve_constants(&extern_items.constant_ids, krate, &collection);

        if !cli.quiet {
            eprintln!("Resolved {} function(s) to IR", resolved_fns.len());
            eprintln!("Resolved {} static(s) to IR", resolved_statics.len());
            eprintln!("Resolved {} constant(s) to IR", resolved_constants.len());
        }

        let type_defs = analysis::collect_type_definitions(
            &resolved_fns,
            &resolved_statics,
            krate,
            &collection,
        )?;

        let c_config = match &config {
            config::Config::C(c) => c,
            _ => anyhow::bail!("Only C output is currently supported"),
        };

        let mut header = String::new();
        codegen::generate_c_header(
            c_config,
            &type_defs,
            &resolved_constants,
            &resolved_fns,
            &resolved_statics,
            &krate.core.krate.index,
            &mut header,
        );

        if let Some(ref output_path) = cli.output {
            fs_err::write(output_path, &header)?;
        } else {
            print!("{header}");
        }
    }

    Ok(())
}
