use std::path::{Path, PathBuf};

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

/// Finds workspace members matching the [`ResolvedInput`].
///
/// A [`ResolvedInput::SingleCrate`] returns exactly one package; a
/// [`ResolvedInput::Directory`] returns every member whose path is under that directory.
pub(super) fn select_packages(
    resolved_input: &ResolvedInput,
    workspace: &guppy::graph::Workspace<'_>,
) -> anyhow::Result<Vec<(guppy::PackageId, String)>> {
    match resolved_input {
        ResolvedInput::SingleCrate(dir) => {
            let dir = dir.canonicalize()?;
            let dir = camino::Utf8PathBuf::try_from(dir)?;
            let relative = pathdiff::diff_utf8_paths(&dir, workspace.root())
                .expect("Failed to compute relative path to target crate");
            let pkg = workspace.member_by_path(&relative).map_err(|e| {
                anyhow::anyhow!("Could not find workspace member for {relative}: {e}")
            })?;
            Ok(vec![(pkg.id().clone(), pkg.name().to_string())])
        }
        ResolvedInput::Directory(dir) => {
            let dir = dir.canonicalize()?;
            let dir = camino::Utf8PathBuf::try_from(dir)?;
            let relative_dir = pathdiff::diff_utf8_paths(&dir, workspace.root())
                .expect("Failed to compute relative path to directory");
            let packages: Vec<_> = workspace
                .iter_by_path()
                .filter(|(path, _)| path.starts_with(&relative_dir))
                .map(|(_, pkg)| (pkg.id().clone(), pkg.name().to_string()))
                .collect();
            anyhow::ensure!(
                !packages.is_empty(),
                "No workspace members found under {}",
                dir
            );
            Ok(packages)
        }
    }
}
