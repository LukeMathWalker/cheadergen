use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use clap::{ArgAction, Parser};
use guppy::graph::DependencyDirection;

use crate::analysis::ExternItemCoordinates;
use crate::config::{Language, PackageConfig, PackageTypeMode, Style};
use crate::diagnostic::{DiagnosticSink, render_diagnostics};
use crate::{analysis, codegen, config, metadata, topological_sort};

use super::input::{PackageSelection, resolve_input, select_packages};
use crate::Collection;

#[derive(Debug, Parser)]
pub(super) struct GenerateArgs {
    /// Path to a directory or Cargo.toml. A Cargo.toml selects a single crate;
    /// a directory selects all workspace members inside it (defaults to current directory).
    input: Option<PathBuf>,

    #[command(flatten)]
    package_selection: PackageSelection,

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

/// Resolved per-package type overrides, keyed by [`guppy::PackageId`].
///
/// Built from the `[package.*]` config sections after resolving crate names
/// (and optional `@version` specifiers) against the dependency graph.
pub(crate) struct PackageTypeOverrides {
    /// Types from these packages are emitted as opaque forward declarations.
    pub opaque: HashSet<guppy::PackageId>,
    /// Types from these packages are not emitted at all.
    pub skipped: HashSet<guppy::PackageId>,
}

/// Parse a package config key into `(name, optional_version_req)`.
///
/// Keys follow the Cargo `name@version` convention:
/// - `"my-dep"` → `("my-dep", None)`
/// - `"foo@1.0"` → `("foo", Some(VersionReq::parse("1.0")?))`
fn parse_package_key(key: &str) -> Result<(&str, Option<guppy::VersionReq>), config::ConfigError> {
    if let Some((name, version_str)) = key.split_once('@') {
        let req = guppy::VersionReq::parse(version_str).map_err(|e| config::ConfigError {
            message: format!(
                "invalid version requirement `{version_str}` in [package.\"{key}\"]: {e}"
            ),
        })?;
        Ok((name, Some(req)))
    } else {
        Ok((key, None))
    }
}

/// Resolve `[package.*]` config entries against the dependency graph.
///
/// Returns a [`PackageTypeOverrides`] containing the resolved package IDs,
/// or an error if any package name is unknown or ambiguous.
fn resolve_package_overrides(
    package_configs: &std::collections::HashMap<String, PackageConfig>,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Result<PackageTypeOverrides, anyhow::Error> {
    let mut opaque = HashSet::new();
    let mut skipped = HashSet::new();
    let graph = collection.package_graph();

    for (key, config) in package_configs {
        let (name, version_req) = parse_package_key(key)?;
        let package_set = graph.resolve_package_name(name);

        if package_set.is_empty() {
            diagnostics
                .error(format!("package `{name}` not found in the dependency graph"))
                .emit();
            continue;
        }

        // Collect matching packages, filtering by version if specified.
        let matching: Vec<_> = package_set
            .packages(DependencyDirection::Forward)
            .filter(|pkg| match &version_req {
                Some(req) => req.matches(pkg.version()),
                None => true,
            })
            .collect();

        if matching.is_empty() {
            diagnostics
                .error(format!(
                    "no version of `{name}` matches requirement `{}`",
                    version_req.as_ref().unwrap()
                ))
                .emit();
            continue;
        }

        // Bare name with multiple versions → error requiring disambiguation.
        if version_req.is_none() && matching.len() > 1 {
            let versions: Vec<_> = matching.iter().map(|p| format!("v{}", p.version())).collect();
            diagnostics
                .error(format!(
                    "package name `{name}` is ambiguous: matches {}; \
                     use [package.\"{name}@<version>\"] to disambiguate",
                    versions.join(" and ")
                ))
                .emit();
            continue;
        }

        let target = match config.types {
            PackageTypeMode::Opaque => &mut opaque,
            PackageTypeMode::Skip => &mut skipped,
        };
        for pkg in &matching {
            target.insert(pkg.id().clone());
        }
    }

    Ok(PackageTypeOverrides { opaque, skipped })
}

/// Entry point for the `generate` subcommand — validates CLI args, loads
/// config and cargo metadata, then iterates over the selected crates.
pub(super) fn generate(cli: &GenerateArgs) -> anyhow::Result<()> {
    if cli.no_header && cli.symbol_file.is_none() {
        anyhow::bail!("--no-header requires --symbol-file");
    }

    let resolved_input = cli.input.as_ref().map(|p| resolve_input(p)).transpose()?;

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

    let metadata_dir = resolved_input
        .as_ref()
        .map(|r| r.dir().clone())
        .unwrap_or_else(|| PathBuf::from("."));
    let package_graph = metadata::load_package_graph(cli.metadata.as_ref(), Some(&metadata_dir))?;
    let packages = select_packages(
        resolved_input.as_ref(),
        &cli.package_selection,
        &package_graph.workspace(),
    )?;

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
    let debug = std::env::var("CHEADERGEN_DEBUG").is_ok_and(|v| v == "true" || v == "1");
    let mut diagnostics = DiagnosticSink::new(ws_root, debug);

    // Resolve per-package overrides against the dependency graph.
    let package_configs = match &config {
        config::Config::C(c) => &c.common.package_configs,
        config::Config::Cxx(c) => &c.common.package_configs,
    };
    let type_overrides = resolve_package_overrides(package_configs, &collection, &mut diagnostics)?;

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
            &type_overrides,
            &mut diagnostics,
        ) {
            Ok(symbols) => {
                all_symbols.extend(symbols);
            }
            Err(e) => {
                diagnostics
                    .error(format!("failed to generate header for `{package_name}`"))
                    .with_error_chain(e.as_ref())
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
        let has_hidden_causes = diagnostics.has_hidden_causes();
        let all = diagnostics.drain();
        let use_color = std::env::var("NO_COLOR").is_err();
        let rendered = render_diagnostics(&all, use_color);
        eprint!("{rendered}");

        if !debug && has_hidden_causes {
            eprintln!("note: rerun with `CHEADERGEN_DEBUG=true` for more details");
        } else {
            eprintln!();
        }

        if all
            .iter()
            .any(|d| d.severity == crate::diagnostic::Severity::Error)
        {
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
    overrides: &PackageTypeOverrides,
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

    let extern_items =
        ExternItemCoordinates::collect(collection, package_id).map_err(|e| anyhow::anyhow!(e))?;

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

        let extern_items = extern_items.resolve(collection, &c_config.common, diagnostics);

        if !cli.quiet {
            eprintln!("Resolved {} function(s) to IR", extern_items.fns.len());
            eprintln!("Resolved {} static(s) to IR", extern_items.statics.len());
            eprintln!(
                "Resolved {} constant(s) to IR",
                extern_items.constants.len()
            );
        }

        let mut type_defs = analysis::collect_type_definitions(
            &extern_items,
            collection,
            c_config.enum_prefix_with_name,
            overrides,
            diagnostics,
        )?;

        // First, establish a baseline source order (type_defs come from a
        // HashMap and have no inherent order). Then apply topological sort
        // to reorder compounds so by-value dependencies are defined first.
        analysis::sort_by_key(&mut type_defs, config::SortKey::SourceOrder, collection);
        topological_sort::topological_sort(&mut type_defs, collection, diagnostics);

        let assoc_constants =
            analysis::find_assoc_constants(&type_defs, krate, collection, diagnostics);

        let mut header = String::new();
        codegen::generate_c_header(
            c_config,
            &type_defs,
            &extern_items.constants,
            &assoc_constants,
            &extern_items.fns,
            &extern_items.statics,
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
