use std::collections::BTreeSet;
use std::path::PathBuf;

use clap::{ArgAction, Parser};

use crate::config::{Language, Style};
use crate::diagnostic::{DiagnosticSink, render_diagnostics};
use crate::{analysis, codegen, config, metadata, topological_sort};

use crate::Collection;
use super::input::{ResolvedInput, resolve_input, select_packages};

#[derive(Debug, Parser)]
pub(super) struct GenerateArgs {
    /// Path to a directory or Cargo.toml. A Cargo.toml selects a single crate;
    /// a directory selects all workspace members inside it (defaults to current directory).
    input: Option<PathBuf>,

    /// Increase verbosity (can be repeated: -v, -vv, -vvv).
    #[arg(short, action = ArgAction::Count)]
    verbose: u8,

    /// Suppress all output.
    #[arg(short, long)]
    quiet: bool,

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

    /// Output directory path.
    #[arg(short, long = "output-dir")]
    output_dir: PathBuf,

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

/// Entry point for the `generate` subcommand — validates CLI args, loads
/// config and cargo metadata, then iterates over the selected crates.
pub(super) fn generate(cli: &GenerateArgs) -> anyhow::Result<()> {
    if cli.no_header && cli.symbol_file.is_none() {
        anyhow::bail!("--no-header requires --symbol-file");
    }

    let resolved_input = match &cli.input {
        Some(p) => resolve_input(p)?,
        None => ResolvedInput::Directory(PathBuf::from(".")),
    };

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

    let package_graph =
        metadata::load_package_graph(cli.metadata.as_ref(), Some(resolved_input.dir()))?;
    let packages = select_packages(&resolved_input, &package_graph.workspace())?;

    let toolchain = std::env::var("CHEADERGEN_DOCS_TOOLCHAIN")
        .unwrap_or_else(|_| metadata::DOCS_TOOLCHAIN.to_string());
    if !cli.quiet {
        eprintln!(
            "Generating headers for {} crate(s) using toolchain `{toolchain}`...",
            packages.len()
        );
    }

    let collection = metadata::create_collection(package_graph)?;

    let mut all_symbols = BTreeSet::new();
    let ws_root: PathBuf = collection
        .package_graph()
        .workspace()
        .root()
        .to_path_buf()
        .into();
    let mut diagnostics = DiagnosticSink::new(ws_root);

    for (package_id, package_name) in &packages {
        if !cli.quiet {
            eprintln!("Generating header for `{package_name}`...");
        }

        match generate_one_crate(
            package_id,
            package_name,
            &config,
            &collection,
            cli,
            &mut diagnostics,
        ) {
            Ok(symbols) => {
                all_symbols.extend(symbols);
            }
            Err(e) => {
                diagnostics
                    .error(format!("failed to generate header for `{package_name}`"))
                    .with_note(format!("{e:?}"))
                    .emit();
            }
        }
    }

    // Write merged symbol file if requested.
    if let Some(ref symbol_file) = cli.symbol_file {
        codegen::write_symbol_file(&all_symbols, symbol_file)?;
    }

    // Render and print diagnostics.
    if !diagnostics.is_empty() {
        let all = diagnostics.drain();
        let use_color = std::env::var("NO_COLOR").is_err();
        let rendered = render_diagnostics(&all, use_color);
        eprint!("{rendered}");

        if all.iter().any(|d| d.severity == crate::diagnostic::Severity::Error) {
            anyhow::bail!("aborting due to previous error(s)");
        }
    }

    Ok(())
}

/// Processes a single crate: loads its rustdoc JSON via [`Collection`],
/// resolves items to the IR, and emits the header file.
fn generate_one_crate(
    package_id: &guppy::PackageId,
    package_name: &str,
    config: &config::Config,
    collection: &Collection,
    cli: &GenerateArgs,
    diagnostics: &mut DiagnosticSink,
) -> anyhow::Result<BTreeSet<String>> {
    let krate = collection
        .get_or_compute(package_id)
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

    let mut extern_items = analysis::find_extern_items(krate);

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

    // Collect symbols for the merged symbol file.
    let symbols = if cli.symbol_file.is_some() {
        analysis::collect_symbols(&extern_items, krate)
    } else {
        BTreeSet::new()
    };

    // Resolve each extern "C" function and static into the IR and generate the header.
    if !cli.no_header {
        let c_config = match config {
            config::Config::C(c) => c,
            _ => anyhow::bail!("Only C output is currently supported"),
        };

        // Sort IDs before resolution — resolvers preserve input order,
        // so the output inherits the sort.
        analysis::sort_local_ids_by_key(
            &mut extern_items.fn_ids,
            c_config.common.fn_sort_by,
            krate,
        );
        analysis::sort_local_ids_by_key(
            &mut extern_items.static_ids,
            c_config.common.static_sort_by,
            krate,
        );
        analysis::sort_local_ids_by_key(
            &mut extern_items.constant_ids,
            c_config.common.constant_sort_by,
            krate,
        );

        let resolved_fns = analysis::resolve_functions(
            &extern_items.fn_ids,
            krate,
            collection,
            diagnostics,
        );
        let resolved_statics = analysis::resolve_statics(
            &extern_items.static_ids,
            krate,
            collection,
            diagnostics,
        );
        let resolved_constants =
            analysis::resolve_constants(&extern_items.constant_ids, krate, collection, diagnostics);

        if !cli.quiet {
            eprintln!("Resolved {} function(s) to IR", resolved_fns.len());
            eprintln!("Resolved {} static(s) to IR", resolved_statics.len());
            eprintln!("Resolved {} constant(s) to IR", resolved_constants.len());
        }

        let annotations = collection.get_annotated_items(package_id);
        let extra_types = analysis::annotated_path_types(annotations, krate, diagnostics);

        let mut type_defs = analysis::collect_type_definitions(
            &resolved_fns,
            &resolved_statics,
            &extra_types,
            collection,
            c_config.enum_prefix_with_name,
            diagnostics,
        )?;

        // First, establish a baseline source order (type_defs come from a
        // HashMap and have no inherent order). Then apply topological sort
        // to reorder compounds so by-value dependencies are defined first.
        analysis::sort_by_key(&mut type_defs, config::SortKey::SourceOrder, collection);
        topological_sort::topological_sort(&mut type_defs, collection, diagnostics);

        let assoc_constants = analysis::find_assoc_constants(&type_defs, krate, collection, diagnostics);

        let mut header = String::new();
        codegen::generate_c_header(
            c_config,
            &type_defs,
            &resolved_constants,
            &assoc_constants,
            &resolved_fns,
            &resolved_statics,
            collection,
            &mut header,
        );

        let filename = format!(
            "{}.{}",
            package_name.replace('-', "_"),
            cli.lang.extension()
        );
        fs_err::create_dir_all(&cli.output_dir)?;
        fs_err::write(cli.output_dir.join(filename), &header)?;
    }

    Ok(symbols)
}
