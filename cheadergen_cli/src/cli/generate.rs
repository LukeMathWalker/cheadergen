use std::collections::{BTreeSet, HashSet};
use std::path::PathBuf;

use crate::diagnostic::Severity;

use clap::{ArgAction, Parser};
use guppy::PackageId;
use guppy::graph::DependencyDirection;

use std::collections::HashMap;

use crate::analysis::ExternItemCoordinates;
use crate::analysis::partitioning::{self, HeaderFilenames, default_header_base_name};
use crate::config::{Language, PackageConfig, PackageTypeMode, Style};
use crate::diagnostic::{DiagnosticSink, render_diagnostics};
use crate::{analysis, codegen, config, metadata, topological_sort};

use super::input::{PackageSelection, filter_library_targets, resolve_input, select_packages};
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

    /// Produce a single combined header per target, inlining all dependency types.
    /// Only valid with a single target package.
    #[arg(long)]
    bundle: bool,

    /// In partitioned mode, delete header files in --output-dir that were not
    /// produced by this run.
    #[arg(long)]
    prune_orphans: bool,

    /// In partitioned mode, skip writing header files that contain no
    /// declarations (no type definitions, constants, statics, or functions).
    /// Not valid with --bundle.
    #[arg(long, conflicts_with = "bundle")]
    skip_empty: bool,
}

/// Resolved per-package type overrides, keyed by [`guppy::PackageId`].
///
/// Built from the `[package.*]` config sections after resolving crate names
/// (and optional `@version` specifiers) against the dependency graph. Also
/// carries the global default for `usize_is_size_t` so the resolver method
/// can layer per-package over global in one place.
pub(crate) struct PackageTypeOverrides {
    /// Types from these packages are emitted as opaque forward declarations.
    pub opaque: HashSet<guppy::PackageId>,
    /// Types from these packages are not emitted at all.
    pub skipped: HashSet<guppy::PackageId>,
    /// Per-package `usize_is_size_t` overrides. Items defined in a package
    /// listed here use the bool here instead of [`Self::global_usize_is_size_t`].
    pub usize_is_size_t: HashMap<guppy::PackageId, bool>,
    /// Global `usize_is_size_t` resolved from the top-level config.
    pub global_usize_is_size_t: bool,
}

impl PackageTypeOverrides {
    /// Resolve the effective `usize_is_size_t` for an item defined in `package_id`.
    ///
    /// Layered lookup (most-specific first): per-package override → global default.
    /// A future per-item annotation layer will be added as the new first step.
    pub fn usize_is_size_t(&self, package_id: &guppy::PackageId) -> bool {
        self.usize_is_size_t
            .get(package_id)
            .copied()
            .unwrap_or(self.global_usize_is_size_t)
    }
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
    package_configs: &HashMap<String, PackageConfig>,
    global_usize_is_size_t: bool,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Result<PackageTypeOverrides, anyhow::Error> {
    let mut opaque = HashSet::new();
    let mut skipped = HashSet::new();
    let mut usize_is_size_t: HashMap<guppy::PackageId, bool> = HashMap::new();
    let graph = collection.package_graph();

    for (key, config) in package_configs {
        let (name, version_req) = parse_package_key(key)?;
        let package_set = graph.resolve_package_name(name);

        if package_set.is_empty() {
            diagnostics
                .error(format!(
                    "package `{name}` not found in the dependency graph"
                ))
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
            let versions: Vec<_> = matching
                .iter()
                .map(|p| format!("v{}", p.version()))
                .collect();
            diagnostics
                .error(format!(
                    "package name `{name}` is ambiguous: matches {}; \
                     use [package.\"{name}@<version>\"] to disambiguate",
                    versions.join(" and ")
                ))
                .emit();
            continue;
        }

        if let Some(types) = config.types {
            let target = match types {
                PackageTypeMode::Opaque => &mut opaque,
                PackageTypeMode::Skip => &mut skipped,
            };
            for pkg in &matching {
                target.insert(pkg.id().clone());
            }
        }
        if let Some(value) = config.usize_is_size_t {
            for pkg in &matching {
                usize_is_size_t.insert(pkg.id().clone(), value);
            }
        }
    }

    for id in opaque.intersection(&skipped) {
        diagnostics
            .error(format!(
                "package `{}` is configured with both `types = \"opaque\"` and `types = \"skip\"`; \
                 pick one",
                id.repr()
            ))
            .emit();
    }

    Ok(PackageTypeOverrides {
        opaque,
        skipped,
        usize_is_size_t,
        global_usize_is_size_t,
    })
}

/// Resolve `[package.<name>] header_name = ...` entries against the dependency
/// graph, producing a `PackageId`-keyed map suitable for [`HeaderFilenames`].
fn resolve_header_renames(
    renames: &HashMap<String, String>,
    collection: &Collection,
    diagnostics: &mut DiagnosticSink,
) -> Result<HashMap<PackageId, String>, anyhow::Error> {
    let graph = collection.package_graph();
    let mut resolved = HashMap::new();

    for (key, header_name) in renames {
        let (name, version_req) = parse_package_key(key)?;
        let package_set = graph.resolve_package_name(name);

        if package_set.is_empty() {
            diagnostics
                .error(format!(
                    "package `{name}` (from `header_name` rename) not found in the \
                     dependency graph"
                ))
                .emit();
            continue;
        }

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
                    "no version of `{name}` matches requirement `{}` (from `header_name` rename)",
                    version_req.as_ref().unwrap()
                ))
                .emit();
            continue;
        }

        if version_req.is_none() && matching.len() > 1 {
            let versions: Vec<_> = matching
                .iter()
                .map(|p| format!("v{}", p.version()))
                .collect();
            diagnostics
                .error(format!(
                    "package name `{name}` is ambiguous for `header_name` rename: matches {}; \
                     use [package.\"{name}@<version>\"] to disambiguate",
                    versions.join(" and ")
                ))
                .emit();
            continue;
        }

        for pkg in &matching {
            resolved.insert(pkg.id().clone(), header_name.clone());
        }
    }

    Ok(resolved)
}

/// Final on-disk base name for a target header (before version disambiguation,
/// which only affects deps in practice). Returns the rename override if set,
/// otherwise the default base name.
fn target_base_name(
    graph: &guppy::graph::PackageGraph,
    pkg_id: &PackageId,
    fallback_name: &str,
    renames: &HashMap<PackageId, String>,
) -> String {
    renames
        .get(pkg_id)
        .cloned()
        .or_else(|| default_header_base_name(graph, pkg_id))
        .unwrap_or_else(|| fallback_name.replace('-', "_"))
}

/// Format the crate this header was generated from, for use as the
/// `<path>` placeholder in the default `autogen_warning` message.
///
/// For workspace members, returns the crate root relative to the workspace
/// root. When the crate sits at the workspace root itself, the relative path
/// would be empty — substitute the crate name so readers see something
/// meaningful instead of a bare `.`. For non-workspace dependencies
/// (registry/git/etc.), the absolute path is per-machine, so fall back to
/// `name@version` to keep the generated header portable. The return type is
/// `String` rather than a path because both fallbacks are package
/// identifiers, not filesystem locations.
fn crate_origin(graph: &guppy::graph::PackageGraph, pkg_id: &PackageId) -> String {
    let Ok(meta) = graph.metadata(pkg_id) else {
        return pkg_id.repr().to_string();
    };
    if !meta.in_workspace() {
        return format!("{}@{}", meta.name(), meta.version());
    }
    let ws_root = graph.workspace().root();
    let crate_root = meta.manifest_path().parent().unwrap_or(ws_root);
    match pathdiff::diff_utf8_paths(crate_root, ws_root) {
        Some(rel) if rel.as_str().is_empty() || rel.as_str() == "." => meta.name().to_string(),
        Some(rel) => rel.into_string(),
        None => crate_root.to_string(),
    }
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
    let mut config_set = raw_config.into_config(&cli.lang, &overrides)?;

    // CLI --bundle overrides the config file setting.
    if cli.bundle {
        config_set.bundle = true;
    }

    if cli.skip_empty && config_set.bundle {
        anyhow::bail!(
            "--skip-empty is only valid in partitioned mode; remove --bundle to use it"
        );
    }

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

    let ws_root: PathBuf = package_graph.workspace().root().to_path_buf().into();
    let debug = std::env::var("CHEADERGEN_DEBUG").is_ok_and(|v| v == "true" || v == "1");
    let mut diagnostics = DiagnosticSink::new(ws_root, debug);

    // Drop packages without a library target. Explicit `-p` selections
    // produce an error; implicit ones (directory / workspace defaults) are
    // skipped with a warning so bin-only siblings don't block the real libs.
    let explicit_names: HashSet<String> =
        cli.package_selection.packages.iter().cloned().collect();
    let packages =
        filter_library_targets(packages, &package_graph, &explicit_names, &mut diagnostics);

    if packages.is_empty() {
        return render_diagnostics_or_bail(&mut diagnostics, debug);
    }

    if config_set.bundle && packages.len() > 1 {
        anyhow::bail!(
            "--bundle is only valid with a single target package, but {} were selected",
            packages.len()
        );
    }

    if cli.prune_orphans && config_set.bundle {
        anyhow::bail!(
            "--prune-orphans is only valid in partitioned mode; remove --bundle to use it"
        );
    }

    let toolchain = std::env::var("CHEADERGEN_DOCS_TOOLCHAIN")
        .unwrap_or_else(|_| metadata::DOCS_TOOLCHAIN.to_string());
    if !cli.quiet {
        eprintln!(
            "Generating headers for {} crate(s) using toolchain `{toolchain}`...",
            packages.len()
        );
    }

    let collection = metadata::create_collection(package_graph)?;

    // Batch-compute rustdoc JSON for all target crates up front. The underlying
    // processor issues a single `cargo doc -p crate1 -p crate2 ...` invocation
    // (chunking only on name collisions), sharing dep compilation across
    // targets. Subsequent per-crate `get_or_compute` calls then hit the cache.
    collection
        .compute_batch(packages.iter().map(|(id, _)| id.clone()))
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut all_symbols = BTreeSet::new();

    // Resolve per-package overrides against the dependency graph.
    // Package overrides are global (not per-header), so resolve once.
    let (package_configs, global_usize_is_size_t) = match &config_set.default {
        config::Config::C(c) => (&c.common.package_configs, c.common.usize_is_size_t),
        config::Config::Cxx(c) => (&c.common.package_configs, c.common.usize_is_size_t),
    };
    let type_overrides = resolve_package_overrides(
        package_configs,
        global_usize_is_size_t,
        &collection,
        &mut diagnostics,
    )?;
    let header_renames =
        resolve_header_renames(&config_set.header_renames, &collection, &mut diagnostics)?;

    // Warn for [header.<name>] sections whose key doesn't match any selected
    // package's final base header name.
    let package_base_names: std::collections::HashSet<String> = packages
        .iter()
        .map(|(id, name)| target_base_name(collection.package_graph(), id, name, &header_renames))
        .collect();
    for header_name in config_set.header_names() {
        if !package_base_names.contains(header_name) {
            diagnostics
                .warning(format!(
                    "`[header.\"{header_name}\"]` in config does not match any selected \
                     package's generated header name"
                ))
                .emit();
        }
    }

    if config_set.bundle {
        // Bundle mode: standalone headers.
        for (package_id, package_name) in &packages {
            if !cli.quiet {
                eprintln!("Generating header for `{package_name}`...");
            }

            let base_name = target_base_name(
                collection.package_graph(),
                package_id,
                package_name,
                &header_renames,
            );
            let config = config_set.for_header(&base_name);

            match generate_one_crate(
                package_id,
                package_name,
                config,
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
    } else {
        // Partitioned mode: generate one header per crate with #include directives.
        match generate_partitioned(
            &packages,
            &config_set,
            &header_renames,
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
                    .error("failed to generate partitioned headers".to_string())
                    .with_error_chain(e.as_ref())
                    .emit();
            }
        }
    }

    // Write merged symbol file if requested.
    if let Some(ref symbol_file) = cli.symbol_file {
        codegen::write_symbol_file(&all_symbols, symbol_file)?;
    }

    render_diagnostics_or_bail(&mut diagnostics, debug)
}

/// Print any pending diagnostics and, if any of them is an error, bail.
fn render_diagnostics_or_bail(diagnostics: &mut DiagnosticSink, debug: bool) -> anyhow::Result<()> {
    if diagnostics.is_empty() {
        return Ok(());
    }
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

    if all.iter().any(|d| d.severity == Severity::Error) {
        anyhow::bail!("aborting due to previous error(s)");
    }
    Ok(())
}

/// Partitioned mode: generates one header per crate with `#include` directives
/// linking them. Types are partitioned by defining crate, generic instantiations
/// go in the consuming crate's header with `#ifndef` guards.
fn generate_partitioned(
    packages: &[(PackageId, String)],
    config_set: &config::ConfigSet,
    header_renames: &HashMap<PackageId, String>,
    collection: &Collection,
    cli: &GenerateArgs,
    type_overrides: &PackageTypeOverrides,
    diagnostics: &mut DiagnosticSink,
) -> anyhow::Result<BTreeSet<String>> {
    let mut all_symbols = BTreeSet::new();
    let graph = collection.package_graph();

    // Step 1: Collect and resolve extern items for each target.
    let mut target_extern_items: Vec<(PackageId, analysis::extern_items::ExternItems)> = Vec::new();

    for (package_id, package_name) in packages {
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

        let coordinates = ExternItemCoordinates::collect(collection, package_id, diagnostics)
            .map_err(|e| anyhow::anyhow!(e))?;

        // Collect symbols for the merged symbol file.
        if cli.symbol_file.is_some() {
            all_symbols.extend(analysis::collect_symbols(&coordinates, krate));
        }

        let base_name = target_base_name(graph, package_id, package_name, header_renames);
        let config = config_set.for_header(&base_name);
        let c_config = match config {
            config::Config::C(c) => c,
            _ => anyhow::bail!("Only C output is currently supported"),
        };

        let extern_items =
            coordinates.resolve(collection, &c_config.common, type_overrides, diagnostics);

        if !cli.quiet {
            eprintln!(
                "`{package_name}`: {} function(s), {} static(s), {} constant(s)",
                extern_items.fns.len(),
                extern_items.statics.len(),
                extern_items.constants.len()
            );
        }

        target_extern_items.push((package_id.clone(), extern_items));
    }

    if cli.no_header {
        return Ok(all_symbols);
    }

    // Step 2: Get a common C config (for enum_prefix_with_name and other shared settings).
    // Use the first target's config as the baseline for type collection settings.
    let first_base_name =
        target_base_name(graph, &packages[0].0, &packages[0].1, header_renames);
    let first_config = config_set.for_header(&first_base_name);
    let c_config = match first_config {
        config::Config::C(c) => c,
        _ => anyhow::bail!("Only C output is currently supported"),
    };

    // Step 3: Unified type collection across all targets.
    let all_type_defs = analysis::collect_type_definitions_multi(
        &target_extern_items,
        collection,
        c_config.enum_prefix_with_name,
        type_overrides,
        diagnostics,
    )?;

    if !cli.quiet {
        eprintln!(
            "Collected {} type definitions across all targets",
            all_type_defs.len()
        );
    }

    // Step 4: Partition types into per-crate buckets.
    let partitioned =
        partitioning::partition_types(all_type_defs, &target_extern_items, type_overrides);

    // Step 5: Build header filename map from the package graph.
    let all_header_pkg_ids: Vec<&PackageId> = partitioned.per_crate.keys().collect();
    let filenames = HeaderFilenames::new(&all_header_pkg_ids, graph, header_renames)
        .map_err(|e| anyhow::anyhow!(e))?;

    // Step 6: Compute include graph.
    let header_deps = partitioning::compute_header_deps(
        &partitioned,
        &target_extern_items,
        type_overrides,
        &filenames,
        cli.lang.extension(),
    );

    // Step 7: Generate each header.
    let target_ids: HashSet<&PackageId> = packages.iter().map(|(id, _)| id).collect();
    // Only use #ifndef guards when there are multiple output headers.
    let multi_header = partitioned.per_crate.len() > 1;

    fs_err::create_dir_all(&cli.output_dir)?;

    let mut written: HashSet<String> = HashSet::new();

    for (pkg_id, mut type_defs) in partitioned.per_crate {
        let is_target = target_ids.contains(&pkg_id);

        // Determine the config for this header by its final on-disk base name.
        let config = config_set.for_header(filenames.base_name(&pkg_id));
        let c_cfg = match config {
            config::Config::C(c) => c,
            _ => continue,
        };

        // Add forward declarations from header deps.
        let deps = header_deps.get(&pkg_id);
        if let Some(deps) = deps {
            for fwd in &deps.forward_decls {
                type_defs.push(fwd.clone());
            }
        }

        // Get extern items for target packages (empty for non-targets).
        let (fns, statics, constants) = if is_target {
            let items = target_extern_items
                .iter()
                .find(|(id, _)| *id == pkg_id)
                .map(|(_, items)| items)
                .unwrap();
            (&items.fns[..], &items.statics[..], &items.constants[..])
        } else {
            (&[][..], &[][..], &[][..])
        };

        // With --skip-empty, headers with no declarations are not written and
        // therefore not added to `written`, so --prune-orphans treats any
        // pre-existing copy as an orphan.
        if cli.skip_empty
            && type_defs.is_empty()
            && fns.is_empty()
            && statics.is_empty()
            && constants.is_empty()
        {
            continue;
        }

        // Sort types: source order first, then topological sort.
        analysis::sort_by_key(&mut type_defs, config::SortKey::SourceOrder, collection);
        topological_sort::topological_sort(&mut type_defs, collection, diagnostics);

        // Find associated constants for types in this header.
        let krate_data = collection
            .get_or_compute(&pkg_id)
            .map_err(|e| anyhow::anyhow!(e))?;
        let assoc_constants =
            analysis::find_assoc_constants(&type_defs, krate_data, collection, diagnostics);

        let dep_includes = deps.map(|d| &d.includes[..]).unwrap_or(&[]);
        let type_hints = deps.map(|d| &d.type_hints[..]).unwrap_or(&[]);

        let mut header = String::new();
        let origin = crate_origin(graph, &pkg_id);
        codegen::generate_c_header(
            c_cfg,
            &type_defs,
            constants,
            &assoc_constants,
            fns,
            statics,
            dep_includes,
            type_hints,
            &type_overrides.skipped,
            multi_header,
            collection,
            &origin,
            &mut header,
        );

        let filename = filenames.filename(&pkg_id, cli.lang.extension());
        fs_err::write(cli.output_dir.join(&filename), &header)?;

        if !cli.quiet {
            let kind = if is_target { "target" } else { "dependency" };
            eprintln!("Wrote {kind} header: {filename}");
        }

        written.insert(filename);
    }

    if cli.prune_orphans {
        prune_orphan_headers(&cli.output_dir, cli.lang.extension(), &written, cli.quiet)?;
    }

    Ok(all_symbols)
}

/// Delete top-level files in `output_dir` whose extension matches
/// `lang_extension` and whose name is not in `keep`. Logs each removal to
/// stderr unless `quiet` is set.
fn prune_orphan_headers(
    output_dir: &std::path::Path,
    lang_extension: &str,
    keep: &HashSet<String>,
    quiet: bool,
) -> anyhow::Result<()> {
    let target_ext = std::ffi::OsStr::new(lang_extension);
    for entry in fs_err::read_dir(output_dir)? {
        let entry = entry?;
        if !entry.file_type()?.is_file() {
            continue;
        }
        let path = entry.path();
        if path.extension() != Some(target_ext) {
            continue;
        }
        let Some(name) = path.file_name().and_then(|n| n.to_str()) else {
            continue;
        };
        if keep.contains(name) {
            continue;
        }
        let name = name.to_string();
        fs_err::remove_file(&path)?;
        if !quiet {
            eprintln!("Removed orphan header: {name}");
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

    let extern_items = ExternItemCoordinates::collect(collection, package_id, diagnostics)
        .map_err(|e| anyhow::anyhow!(e))?;

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

        let extern_items = extern_items.resolve(collection, &c_config.common, overrides, diagnostics);

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
        let origin = crate_origin(collection.package_graph(), package_id);
        codegen::generate_c_header(
            c_config,
            &type_defs,
            &extern_items.constants,
            &assoc_constants,
            &extern_items.fns,
            &extern_items.statics,
            &[],
            &[],
            &overrides.skipped,
            false,
            collection,
            &origin,
            &mut header,
        );

        let base = default_header_base_name(collection.package_graph(), package_id)
            .unwrap_or_else(|| package_name.replace('-', "_"));
        let filename = format!("{}.{}", base, cli.lang.extension());
        fs_err::create_dir_all(&cli.output_dir)?;
        fs_err::write(cli.output_dir.join(filename), &header)?;
    }

    Ok(symbols)
}
