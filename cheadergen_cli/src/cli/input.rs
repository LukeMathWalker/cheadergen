use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use guppy::PackageId;

#[derive(Debug, Clone, Default, clap::Args)]
pub(super) struct PackageSelection {
    /// Select specific workspace member(s) by name (repeatable).
    #[arg(short = 'p', long = "package")]
    pub packages: Vec<String>,

    /// Exclude workspace member(s) by name (repeatable).
    #[arg(long = "exclude")]
    pub exclude: Vec<String>,
}

pub(super) enum ResolvedInput {
    /// User pointed at a specific Cargo.toml — single crate.
    SingleCrate(PathBuf),
    /// User pointed at a directory — select all workspace members inside it.
    Directory(PathBuf),
}

impl ResolvedInput {
    /// Returns the directory path regardless of variant.
    pub(super) fn dir(&self) -> &PathBuf {
        match self {
            ResolvedInput::SingleCrate(p) | ResolvedInput::Directory(p) => p,
        }
    }
}

/// Classifies a user-provided path as a [`ResolvedInput::SingleCrate`] (if it
/// points at a `Cargo.toml`) or a [`ResolvedInput::Directory`].
pub(super) fn resolve_input(input: &Path) -> anyhow::Result<ResolvedInput> {
    if input.file_name() == Some("Cargo.toml".as_ref()) {
        anyhow::ensure!(
            input.try_exists()?,
            "Cargo.toml not found at {}",
            input.display()
        );
        let parent = input.parent().unwrap();
        if parent.as_os_str().is_empty() {
            Ok(ResolvedInput::SingleCrate(PathBuf::from(".")))
        } else {
            Ok(ResolvedInput::SingleCrate(parent.to_path_buf()))
        }
    } else if input.is_dir() {
        Ok(ResolvedInput::Directory(input.to_path_buf()))
    } else {
        anyhow::bail!(
            "input must be a directory or a path to a Cargo.toml file, got: {}",
            input.display()
        );
    }
}

/// Finds workspace members matching the [`ResolvedInput`] and [`PackageSelection`].
///
/// 1. Path-based set from `resolved_input` (if provided).
/// 2. Add any `-p`/`--package` names.
/// 3. Remove any `--exclude` names.
/// 4. Error if the final set is empty.
pub(super) fn select_packages(
    resolved_input: Option<&ResolvedInput>,
    selection: &PackageSelection,
    workspace: &guppy::graph::Workspace<'_>,
) -> anyhow::Result<Vec<(PackageId, String)>> {
    // Keyed by PackageId to deduplicate.
    let mut selected: BTreeMap<PackageId, String> = BTreeMap::new();

    // 1. Path-based set.
    match resolved_input {
        Some(ResolvedInput::SingleCrate(dir)) => {
            let dir = dir.canonicalize()?;
            let dir = camino::Utf8PathBuf::try_from(dir)?;
            let relative = pathdiff::diff_utf8_paths(&dir, workspace.root())
                .expect("Failed to compute relative path to target crate");
            let pkg = workspace.member_by_path(&relative).map_err(|e| {
                anyhow::anyhow!("Could not find workspace member for {relative}: {e}")
            })?;
            selected.insert(pkg.id().clone(), pkg.name().to_string());
        }
        Some(ResolvedInput::Directory(dir)) => {
            let dir = dir.canonicalize()?;
            let dir = camino::Utf8PathBuf::try_from(dir)?;
            let relative_dir = pathdiff::diff_utf8_paths(&dir, workspace.root())
                .expect("Failed to compute relative path to directory");
            for (path, pkg) in workspace.iter_by_path() {
                if path.starts_with(&relative_dir) {
                    selected.insert(pkg.id().clone(), pkg.name().to_string());
                }
            }
        }
        None if selection.packages.is_empty() => {
            // No path and no -p flags: select all workspace members.
            for pkg in workspace.iter() {
                selected.insert(pkg.id().clone(), pkg.name().to_string());
            }
        }
        None => {
            // No path but -p flags provided: start with empty set, packages
            // will be added in step 2.
        }
    }

    // 2. Add `-p`/`--package` names.
    for name in &selection.packages {
        let pkg = workspace
            .member_by_name(name)
            .map_err(|e| anyhow::anyhow!("unknown package `{name}`: {e}"))?;
        selected.insert(pkg.id().clone(), pkg.name().to_string());
    }

    // 3. Remove `--exclude` names.
    for name in &selection.exclude {
        let pkg = workspace
            .member_by_name(name)
            .map_err(|e| anyhow::anyhow!("unknown package in --exclude `{name}`: {e}"))?;
        selected.remove(pkg.id());
    }

    // 4. Final validation.
    anyhow::ensure!(
        !selected.is_empty(),
        "No packages selected (after applying --package and --exclude filters)"
    );

    Ok(selected.into_iter().collect())
}
