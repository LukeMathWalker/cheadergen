use std::collections::{BTreeMap, HashSet};
use std::path::{Path, PathBuf};

use camino::Utf8Path;
use guppy::PackageId;
use guppy::graph::{BuildTargetId, PackageGraph, PackageMetadata, Workspace};

use crate::diagnostic::DiagnosticSink;

#[derive(Debug, Clone, Default, clap::Args)]
pub(super) struct PackageSelection {
    /// Select specific workspace member(s) by name (repeatable).
    #[arg(short = 'p', long = "package")]
    pub packages: Vec<String>,

    /// Exclude workspace member(s) by name (repeatable).
    #[arg(long = "exclude")]
    pub exclude: Vec<String>,
}

/// Finds workspace members matching the provided input directories and
/// [`PackageSelection`].
///
/// 1. Collect inclusions from `--input-dir` (every workspace member under
///    any of the given directories) and `-p`/`--package` (by name).
/// 2. If the inclusion set is empty (no `--input-dir`, no `-p`), fall back
///    to the Cargo-style cwd-aware default (see [`default_selection`]).
/// 3. Remove any `--exclude` names.
/// 4. Error if the final set is empty.
pub(super) fn select_packages(
    input_dirs: &[PathBuf],
    selection: &PackageSelection,
    workspace: &Workspace<'_>,
) -> anyhow::Result<Vec<(PackageId, String)>> {
    // Keyed by PackageId to deduplicate across overlapping inclusions.
    let mut selected: BTreeMap<PackageId, String> = BTreeMap::new();

    // 1a. Path-based inclusions.
    for dir in input_dirs {
        for (id, name) in members_under(dir, workspace)? {
            selected.insert(id, name);
        }
    }

    // 1b. Name-based inclusions.
    for name in &selection.packages {
        let pkg = workspace
            .member_by_name(name)
            .map_err(|e| anyhow::anyhow!("unknown package `{name}`: {e}"))?;
        selected.insert(pkg.id().clone(), pkg.name().to_string());
    }

    // 2. Fall back to the default when nothing was explicitly included.
    if selected.is_empty() {
        for (id, name) in default_selection(workspace)? {
            selected.insert(id, name);
        }
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

/// Cargo-style default selection when neither `--input-dir` nor `-p` is
/// provided.
///
/// Mirrors what `cargo build` (and friends) would do: if cwd sits inside a
/// workspace member's directory, pick that member; otherwise fall back to
/// `[workspace] default-members` if configured, which for a virtual
/// workspace expands to every member and for a non-virtual workspace points
/// at the root package.
fn default_selection(workspace: &Workspace<'_>) -> anyhow::Result<Vec<(PackageId, String)>> {
    let cwd = std::env::current_dir()?.canonicalize()?;
    let cwd = camino::Utf8PathBuf::try_from(cwd)?;

    if let Some(pkg) = current_package(workspace, &cwd) {
        return Ok(vec![(pkg.id().clone(), pkg.name().to_string())]);
    }

    Ok(workspace
        .default_members()
        .map(|p| (p.id().clone(), p.name().to_string()))
        .collect())
}

/// Find the workspace member whose root directory contains `cwd` and sits
/// closest to it.
///
/// "Closest" matters in hybrid workspaces where the root is both a package
/// and a workspace with nested members: cwd inside `crates/alpha/` must
/// resolve to alpha, not the root package — even though both contain cwd.
fn current_package<'g>(workspace: &Workspace<'g>, cwd: &Utf8Path) -> Option<PackageMetadata<'g>> {
    let candidates = workspace
        .iter()
        .filter_map(|pkg| pkg.manifest_path().parent().map(|dir| (dir, pkg)));
    closest_containing(cwd, candidates)
}

/// Pick the candidate whose directory contains `cwd` and sits closest to it
/// (i.e. has the longest path among those that contain `cwd`). Returns
/// `None` if no candidate contains `cwd`.
///
/// Path matching is component-wise (via [`Utf8Path::starts_with`]), so
/// `/a/foo` is not considered to contain `/a/foobar`.
fn closest_containing<'a, T>(
    cwd: &Utf8Path,
    candidates: impl IntoIterator<Item = (&'a Utf8Path, T)>,
) -> Option<T> {
    candidates
        .into_iter()
        .filter(|(dir, _)| cwd.starts_with(dir))
        .max_by_key(|(dir, _)| dir.components().count())
        .map(|(_, value)| value)
}

/// Resolve all workspace members under `dir`, where `dir` must be an
/// existing directory inside `workspace`'s root.
///
/// Returns an error — not an empty vector — if `dir` is outside the
/// workspace or if no workspace member's manifest lives under it. Giving up
/// silently would let typos (e.g. `--input-dir crates/aplha`) produce
/// "selected zero crates" further down without explaining which input went
/// nowhere.
fn members_under(
    dir: &Path,
    workspace: &Workspace<'_>,
) -> anyhow::Result<Vec<(PackageId, String)>> {
    anyhow::ensure!(
        dir.try_exists()?,
        "--input-dir `{}` does not exist",
        dir.display()
    );
    anyhow::ensure!(
        dir.is_dir(),
        "--input-dir `{}` is not a directory",
        dir.display()
    );

    let canonical = dir.canonicalize()?;
    let canonical = camino::Utf8PathBuf::try_from(canonical)?;
    // `relative` may start with `..` when the workspace declares members
    // outside its own root (a `[workspace] members = ["../foo"]` pattern).
    // `iter_by_path` reflects that, so the `starts_with` check below
    // handles both layouts uniformly.
    let relative = pathdiff::diff_utf8_paths(&canonical, workspace.root())
        .expect("Failed to compute relative path to --input-dir");

    let mut found = Vec::new();
    for (path, pkg) in workspace.iter_by_path() {
        if path.starts_with(&relative) {
            found.push((pkg.id().clone(), pkg.name().to_string()));
        }
    }

    anyhow::ensure!(
        !found.is_empty(),
        "--input-dir `{}` does not contain any workspace members",
        dir.display()
    );

    Ok(found)
}

/// Returns `true` if `package_id` has a library build target.
fn has_library_target(graph: &PackageGraph, package_id: &PackageId) -> bool {
    graph
        .metadata(package_id)
        .map(|meta| {
            meta.build_targets()
                .any(|t| matches!(t.id(), BuildTargetId::Library))
        })
        .unwrap_or(false)
}

/// Filter out packages that have no library target.
///
/// cheadergen can only generate headers from library crates, so binary-only
/// packages (e.g. CLI crates in a workspace) must be dropped before rustdoc
/// is invoked.
///
/// Packages listed in `explicit_names` are assumed to have been named by the
/// user (`-p`/`--package`); dropping them produces an error diagnostic because
/// the user asked for something that cannot be done. Packages picked up
/// implicitly (via a directory argument or workspace defaults) produce a
/// warning and are silently skipped.
pub(super) fn filter_library_targets(
    packages: Vec<(PackageId, String)>,
    graph: &PackageGraph,
    explicit_names: &HashSet<String>,
    diagnostics: &mut DiagnosticSink,
) -> Vec<(PackageId, String)> {
    packages
        .into_iter()
        .filter(|(id, name)| {
            if has_library_target(graph, id) {
                return true;
            }
            let msg = format!(
                "package `{name}` has no library target; cheadergen can only generate \
                 headers from library crates"
            );
            if explicit_names.contains(name) {
                diagnostics.error(msg).emit();
            } else {
                diagnostics.warning(format!("{msg}; skipping")).emit();
            }
            false
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use camino::Utf8PathBuf;

    /// Builds owned `Utf8PathBuf`s up front so the iterator can hand out
    /// `&Utf8Path` slices borrowed from this vec.
    fn lookup<'a>(cwd: &str, candidates: &'a [(Utf8PathBuf, &'a str)]) -> Option<&'a str> {
        closest_containing(
            &Utf8PathBuf::from(cwd),
            candidates.iter().map(|(dir, name)| (dir.as_path(), *name)),
        )
    }

    fn dirs<'a>(entries: &[(&'a str, &'a str)]) -> Vec<(Utf8PathBuf, &'a str)> {
        entries
            .iter()
            .map(|(dir, name)| (Utf8PathBuf::from(*dir), *name))
            .collect()
    }

    #[test]
    fn picks_candidate_that_contains_cwd() {
        let c = dirs(&[("/ws/crates/alpha", "alpha")]);
        assert_eq!(lookup("/ws/crates/alpha/src", &c), Some("alpha"));
    }

    #[test]
    fn matches_when_cwd_equals_candidate_root() {
        let c = dirs(&[("/ws/crates/alpha", "alpha")]);
        assert_eq!(lookup("/ws/crates/alpha", &c), Some("alpha"));
    }

    #[test]
    fn returns_none_when_no_candidate_contains_cwd() {
        let c = dirs(&[("/ws/crates/alpha", "alpha"), ("/ws/crates/beta", "beta")]);
        assert_eq!(lookup("/elsewhere", &c), None);
    }

    #[test]
    fn prefers_closest_match_in_hybrid_workspace() {
        // Workspace root `/ws` is itself a package; `/ws/crates/inner` is a
        // nested member. cwd inside the nested member must resolve to it,
        // not to the outer root package.
        let c = dirs(&[("/ws", "root_pkg"), ("/ws/crates/inner", "inner")]);
        assert_eq!(lookup("/ws/crates/inner/src", &c), Some("inner"));
    }

    #[test]
    fn uses_component_prefix_not_string_prefix() {
        // `/ws/foo` must NOT be considered to contain `/ws/foobar` even
        // though the latter starts with the former as a string.
        let c = dirs(&[("/ws/foo", "foo")]);
        assert_eq!(lookup("/ws/foobar", &c), None);
    }

    #[test]
    fn cwd_at_virtual_root_with_nested_members_finds_no_match() {
        // Virtual workspace at `/ws`, members at `/ws/alpha` and `/ws/beta`.
        // cwd at `/ws` is not inside any member — caller falls back to
        // `default_members()` (guppy's responsibility, not tested here).
        let c = dirs(&[("/ws/alpha", "alpha"), ("/ws/beta", "beta")]);
        assert_eq!(lookup("/ws", &c), None);
    }
}
