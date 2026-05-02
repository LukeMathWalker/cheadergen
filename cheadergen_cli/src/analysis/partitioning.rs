use std::collections::{HashMap, HashSet};

use guppy::graph::{BuildTargetId, PackageGraph};
use guppy::{PackageId, Version};
use rustdoc_ir::Type;

use super::type_collection::{CTypeDefinition, CTypeKind, c_type_name};
use crate::analysis::extern_items::{ExternItems, FreeFunctionItem};
use crate::cli::generate::PackageTypeOverrides;
use crate::static_item::StaticItem;

/// The result of partitioning types across per-crate headers.
pub struct PartitionedTypes {
    /// Types assigned to each header file, keyed by package.
    pub per_crate: HashMap<PackageId, Vec<CTypeDefinition>>,
    /// Opaque types (from packages marked `types = "opaque"`).
    /// These are NOT placed in any header but are available for forward declarations.
    pub opaque_types: Vec<CTypeDefinition>,
}

/// Per-header dependency information: which other headers to include
/// and which types to forward-declare inline.
pub struct HeaderDeps {
    /// Bare filenames of dependency crate headers to `#include`.
    pub includes: Vec<String>,
    /// Forward declarations to emit inline (opaque or pointer-only deps).
    pub forward_decls: Vec<CTypeDefinition>,
    /// Type metadata from included deps (rename and tag info).
    /// These are NOT emitted as C code — they only feed the `type_meta` map
    /// so that function signatures in the consuming header use the correct
    /// renamed names and type tags.
    pub type_hints: Vec<CTypeDefinition>,
}

/// Partition a flat list of type definitions into per-crate buckets.
///
/// - Non-generic types go in the header of their defining crate.
/// - Generic instantiations go in the header of the crate that consumes them
///   (the crate whose non-generic types or extern functions reference them).
/// - A generic instantiation may be assigned to multiple headers.
/// - Types from opaque or skipped packages are excluded.
pub fn partition_types(
    type_defs: Vec<CTypeDefinition>,
    target_extern_items: &[(PackageId, ExternItems)],
    overrides: &PackageTypeOverrides,
) -> PartitionedTypes {
    // Separate generic instantiations from non-generic types.
    let mut non_generics: Vec<CTypeDefinition> = Vec::new();
    let mut generics: Vec<CTypeDefinition> = Vec::new();
    let mut opaque_types: Vec<CTypeDefinition> = Vec::new();

    for def in type_defs {
        if overrides.skipped.contains(&def.defining_package) {
            continue;
        }
        if overrides.opaque.contains(&def.defining_package) {
            // Opaque types don't get their own header. Store them separately
            // for forward declaration in consuming headers.
            opaque_types.push(def);
            continue;
        }

        if def.is_generic_instantiation {
            generics.push(def);
        } else {
            non_generics.push(def);
        }
    }

    // Step 1: Assign non-generic types to their defining crate.
    // Ensure every target package has an entry (even if it has no types of its own).
    let mut per_crate: HashMap<PackageId, Vec<CTypeDefinition>> = HashMap::new();
    for (pkg_id, _) in target_extern_items {
        per_crate.entry(pkg_id.clone()).or_default();
    }
    for def in non_generics {
        per_crate
            .entry(def.defining_package.clone())
            .or_default()
            .push(def);
    }

    // Step 2: Assign generic instantiations to consuming crates.
    // Build a lookup from C type name → generic definitions.
    let generic_by_name: HashMap<&str, Vec<usize>> = {
        let mut map: HashMap<&str, Vec<usize>> = HashMap::new();
        for (idx, def) in generics.iter().enumerate() {
            map.entry(def.name.as_str()).or_default().push(idx);
        }
        map
    };

    // Track which generics have been assigned to which packages.
    let mut generic_assignments: HashMap<usize, HashSet<PackageId>> = HashMap::new();

    // Scan non-generic types' fields to find generic consumers.
    for (pkg_id, defs) in &per_crate {
        for def in defs {
            let referenced = collect_by_value_type_names(def);
            for name in &referenced {
                if let Some(indices) = generic_by_name.get(name.as_str()) {
                    for &idx in indices {
                        generic_assignments
                            .entry(idx)
                            .or_default()
                            .insert(pkg_id.clone());
                    }
                }
            }
        }
    }

    // Scan target extern functions/statics for generic consumers.
    for (pkg_id, extern_items) in target_extern_items {
        let fn_type_names = collect_fn_type_names(&extern_items.fns, &extern_items.statics);
        for name in &fn_type_names {
            if let Some(indices) = generic_by_name.get(name.as_str()) {
                for &idx in indices {
                    generic_assignments
                        .entry(idx)
                        .or_default()
                        .insert(pkg_id.clone());
                }
            }
        }
    }

    // Iterate: generics assigned in this round may reference other generics in their fields.
    loop {
        let mut new_assignments = false;
        // Collect current state to avoid borrow issues.
        let snapshot: Vec<(usize, HashSet<PackageId>)> = generic_assignments
            .iter()
            .map(|(&idx, pkgs)| (idx, pkgs.clone()))
            .collect();

        for (idx, pkgs) in &snapshot {
            let def = &generics[*idx];
            let referenced = collect_by_value_type_names(def);
            for name in &referenced {
                if let Some(target_indices) = generic_by_name.get(name.as_str()) {
                    for &target_idx in target_indices {
                        let entry = generic_assignments.entry(target_idx).or_default();
                        for pkg in pkgs {
                            if entry.insert(pkg.clone()) {
                                new_assignments = true;
                            }
                        }
                    }
                }
            }
        }

        if !new_assignments {
            break;
        }
    }

    // Scan for pointer-only references to generics from function signatures
    // and type fields. These need opaque forward declarations in the consuming header.
    for (pkg_id, extern_items) in target_extern_items {
        for func in &extern_items.fns {
            for input in &func.function.header.inputs {
                let mut ptr_names = Vec::new();
                collect_pointer_names_from_type(&input.type_, &mut ptr_names);
                for name in &ptr_names {
                    if let Some(indices) = generic_by_name.get(name.as_str()) {
                        for &idx in indices {
                            generic_assignments
                                .entry(idx)
                                .or_default()
                                .insert(pkg_id.clone());
                        }
                    }
                }
            }
            if let Some(ref output) = func.function.header.output {
                let mut ptr_names = Vec::new();
                collect_pointer_names_from_type(output, &mut ptr_names);
                for name in &ptr_names {
                    if let Some(indices) = generic_by_name.get(name.as_str()) {
                        for &idx in indices {
                            generic_assignments
                                .entry(idx)
                                .or_default()
                                .insert(pkg_id.clone());
                        }
                    }
                }
            }
        }
    }
    // Also scan non-generic types' pointer fields for generic references.
    for (pkg_id, defs) in &per_crate {
        for def in defs {
            let ptr_names = collect_pointer_type_names(def);
            for name in &ptr_names {
                if let Some(indices) = generic_by_name.get(name.as_str()) {
                    for &idx in indices {
                        generic_assignments
                            .entry(idx)
                            .or_default()
                            .insert(pkg_id.clone());
                    }
                }
            }
        }
    }

    // Place generic instantiations into their assigned packages.
    // Generics used by-value get full definitions; those only used behind
    // pointers get opaque forward declarations.
    let by_value_generics: HashSet<usize> = {
        // A generic is by-value if any non-generic type has it as a by-value
        // field, any extern function uses it by-value in its signature, or it
        // is reached transitively through another by-value generic (e.g. a
        // typedef generic `SmallVec<Foo> = Vec2<Foo, 8>` pulls `Vec2<Foo, 8>`
        // in by value).
        let mut by_val: HashSet<usize> = HashSet::new();
        for defs in per_crate.values() {
            for def in defs {
                let names = collect_by_value_type_names(def);
                for name in &names {
                    if let Some(indices) = generic_by_name.get(name.as_str()) {
                        by_val.extend(indices);
                    }
                }
            }
        }
        for (_, extern_items) in target_extern_items {
            let names = collect_fn_type_names(&extern_items.fns, &extern_items.statics);
            for name in &names {
                if let Some(indices) = generic_by_name.get(name.as_str()) {
                    by_val.extend(indices);
                }
            }
        }
        // Transitive closure: a by-value generic's own by-value fields may
        // reference other generics that must themselves be emitted with full
        // definitions.
        loop {
            let mut added = false;
            let snapshot: Vec<usize> = by_val.iter().copied().collect();
            for idx in snapshot {
                let names = collect_by_value_type_names(&generics[idx]);
                for name in &names {
                    if let Some(indices) = generic_by_name.get(name.as_str()) {
                        for &i in indices {
                            if by_val.insert(i) {
                                added = true;
                            }
                        }
                    }
                }
            }
            if !added {
                break;
            }
        }
        by_val
    };

    for (idx, def) in generics.into_iter().enumerate() {
        if let Some(pkgs) = generic_assignments.remove(&idx) {
            let is_by_value = by_value_generics.contains(&idx);
            let pkgs_vec: Vec<PackageId> = pkgs.into_iter().collect();

            for (i, pkg) in pkgs_vec.iter().enumerate() {
                let kind = if is_by_value {
                    def.kind.clone()
                } else {
                    // Pointer-only: emit as opaque forward declaration.
                    forward_decl_kind(&def.kind)
                };

                if i + 1 < pkgs_vec.len() {
                    per_crate
                        .entry(pkg.clone())
                        .or_default()
                        .push(CTypeDefinition {
                            name: def.name.clone(),
                            original_name: def.original_name.clone(),
                            kind,
                            rustdoc_id: def.rustdoc_id.clone(),
                            defining_package: def.defining_package.clone(),
                            is_generic_instantiation: is_by_value,
                            usize_is_size_t: def.usize_is_size_t,
                        });
                } else {
                    per_crate
                        .entry(pkg.clone())
                        .or_default()
                        .push(CTypeDefinition {
                            name: def.name.clone(),
                            original_name: def.original_name.clone(),
                            kind,
                            rustdoc_id: def.rustdoc_id.clone(),
                            defining_package: def.defining_package.clone(),
                            is_generic_instantiation: is_by_value,
                            usize_is_size_t: def.usize_is_size_t,
                        });
                    break;
                }
            }
        }
    }

    // Prune non-target crates that only contain forward-declarable types.
    // Opaque structs/unions and typedefs (aliases, `repr(transparent)` wrappers)
    // can all be expressed inline in consuming headers without a separate dep
    // header — typedefs reference their inner type, which is itself forward-
    // declared or included from elsewhere.
    let target_ids: HashSet<&PackageId> = target_extern_items.iter().map(|(id, _)| id).collect();
    let opaque_only_crates: Vec<PackageId> = per_crate
        .iter()
        .filter(|(id, defs)| {
            !target_ids.contains(id)
                && defs.iter().all(|d| {
                    matches!(
                        d.kind,
                        CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion | CTypeKind::Typedef(_)
                    )
                })
        })
        .map(|(id, _)| id.clone())
        .collect();
    for id in opaque_only_crates {
        if let Some(defs) = per_crate.remove(&id) {
            opaque_types.extend(defs);
        }
    }

    PartitionedTypes {
        per_crate,
        opaque_types,
    }
}

/// Compute per-header dependency information: includes and forward declarations.
///
/// For each header package H, scans its types and extern items to determine:
/// - Which dependency crate headers to `#include` (any type used by-value).
/// - Which types to forward-declare inline (pointer-only or opaque deps).
pub fn compute_header_deps(
    partitioned: &PartitionedTypes,
    target_extern_items: &[(PackageId, ExternItems)],
    overrides: &PackageTypeOverrides,
    filenames: &HeaderFilenames,
    lang_extension: &str,
) -> HashMap<PackageId, HeaderDeps> {
    // Build a set of packages that have headers.
    let packages_with_headers: HashSet<&PackageId> = partitioned.per_crate.keys().collect();

    // Build a lookup: type name → defining package (for non-generic types only).
    let mut type_to_package: HashMap<&str, &PackageId> = HashMap::new();
    for (pkg_id, defs) in &partitioned.per_crate {
        for def in defs {
            if !def.is_generic_instantiation {
                type_to_package.insert(&def.name, pkg_id);
                if let Some(ref orig) = def.original_name {
                    type_to_package.insert(orig, pkg_id);
                }
            }
        }
    }

    let mut result: HashMap<PackageId, HeaderDeps> = HashMap::new();

    for (pkg_id, defs) in &partitioned.per_crate {
        // Collect type names referenced from this header's types (by-value and pointer).
        let mut by_value_from: HashSet<&PackageId> = HashSet::new();
        let mut pointer_only_from: HashSet<&PackageId> = HashSet::new();

        // Scan types in this header.
        for def in defs {
            // By-value references from fields.
            let by_value_names = collect_by_value_type_names(def);
            for name in &by_value_names {
                if let Some(&dep_pkg) = type_to_package.get(name.as_str())
                    && dep_pkg != pkg_id
                {
                    by_value_from.insert(dep_pkg);
                }
            }

            // Pointer references.
            let ptr_names = collect_pointer_type_names(def);
            for name in &ptr_names {
                if let Some(&dep_pkg) = type_to_package.get(name.as_str())
                    && dep_pkg != pkg_id
                {
                    pointer_only_from.insert(dep_pkg);
                }
            }
        }

        // Scan extern functions/statics if this is a target package.
        if let Some((_, extern_items)) = target_extern_items.iter().find(|(id, _)| id == pkg_id) {
            let fn_type_names = collect_fn_type_names(&extern_items.fns, &extern_items.statics);
            for name in &fn_type_names {
                if let Some(&dep_pkg) = type_to_package.get(name.as_str())
                    && dep_pkg != pkg_id
                {
                    by_value_from.insert(dep_pkg);
                }
            }
            for func in &extern_items.fns {
                let mut ptr_names = Vec::new();
                for input in &func.function.header.inputs {
                    collect_pointer_names_from_type(&input.type_, &mut ptr_names);
                }
                if let Some(ref output) = func.function.header.output {
                    collect_pointer_names_from_type(output, &mut ptr_names);
                }
                for name in &ptr_names {
                    if let Some(&dep_pkg) = type_to_package.get(name.as_str())
                        && dep_pkg != pkg_id
                    {
                        pointer_only_from.insert(dep_pkg);
                    }
                }
            }
        }

        // Remove by_value packages from pointer_only (include subsumes forward-decl).
        pointer_only_from.retain(|p| !by_value_from.contains(p));

        // Build includes (only for packages that actually have headers).
        // Pointer-only deps with their own header don't get auto-included —
        // they're forward-declared inline below to keep the consumer header
        // self-contained.
        let mut includes: Vec<String> = by_value_from
            .iter()
            .filter(|p| packages_with_headers.contains(*p) && !overrides.opaque.contains(*p))
            .map(|p| filenames.filename(p, lang_extension))
            .collect();
        includes.sort();

        // Build forward declarations for pointer-only deps and opaque deps.
        let mut forward_decls: Vec<CTypeDefinition> = Vec::new();

        // Opaque deps: forward-declare any opaque type referenced from this header.
        let mut all_referenced: HashSet<String> = defs
            .iter()
            .flat_map(collect_all_referenced_type_names)
            .collect();
        if let Some((_, extern_items)) = target_extern_items.iter().find(|(id, _)| id == pkg_id) {
            all_referenced.extend(collect_fn_type_names(
                &extern_items.fns,
                &extern_items.statics,
            ));
            for func in &extern_items.fns {
                for input in &func.function.header.inputs {
                    let mut ptr_names = Vec::new();
                    collect_pointer_names_from_type(&input.type_, &mut ptr_names);
                    all_referenced.extend(ptr_names);
                }
                if let Some(ref output) = func.function.header.output {
                    let mut ptr_names = Vec::new();
                    collect_pointer_names_from_type(output, &mut ptr_names);
                    all_referenced.extend(ptr_names);
                }
            }
        }
        for opaque_def in &partitioned.opaque_types {
            let matches = all_referenced.contains(&opaque_def.name)
                || opaque_def
                    .original_name
                    .as_ref()
                    .is_some_and(|orig| all_referenced.contains(orig));
            if matches {
                forward_decls.push(opaque_def.clone());
            }
        }

        // Pointer-only deps with their own header: forward-declare the
        // referenced types inline so the consumer header doesn't need to
        // `#include` the dep header (which would drag in unrelated decls).
        // Compound kinds are downgraded via `forward_decl_kind`; typedefs and
        // fieldless enums are emitted in full because C has no forward-decl
        // form for them.
        //
        // `pointer_only_from` is built from `type_to_package`, which only
        // contains types from non-pruned, header-having deps — so every
        // `dep_pkg` here is guaranteed to have an entry in
        // `partitioned.per_crate`. Pointer-only references to *pruned*
        // (typedef/opaque-only) deps are already covered by the
        // `partitioned.opaque_types` matching loop above.
        for dep_pkg in &pointer_only_from {
            let mut ptr_names: HashSet<String> = HashSet::new();
            for def in defs {
                for n in collect_pointer_type_names(def) {
                    if type_to_package.get(n.as_str()) == Some(dep_pkg) {
                        ptr_names.insert(n);
                    }
                }
            }
            if let Some((_, extern_items)) = target_extern_items.iter().find(|(id, _)| id == pkg_id)
            {
                for func in &extern_items.fns {
                    let mut names = Vec::new();
                    for input in &func.function.header.inputs {
                        collect_pointer_names_from_type(&input.type_, &mut names);
                    }
                    if let Some(ref output) = func.function.header.output {
                        collect_pointer_names_from_type(output, &mut names);
                    }
                    for n in names {
                        if type_to_package.get(n.as_str()) == Some(dep_pkg) {
                            ptr_names.insert(n);
                        }
                    }
                }
            }

            let Some(dep_defs) = partitioned.per_crate.get(*dep_pkg) else {
                continue;
            };
            for def in dep_defs {
                let matches = ptr_names.contains(&def.name)
                    || def
                        .original_name
                        .as_ref()
                        .is_some_and(|orig| ptr_names.contains(orig));
                if matches {
                    forward_decls.push(CTypeDefinition {
                        kind: forward_decl_kind(&def.kind),
                        ..def.clone()
                    });
                }
            }
        }

        // Deduplicate forward declarations by name.
        let mut seen_names: HashSet<String> = HashSet::new();
        forward_decls.retain(|d| seen_names.insert(d.name.clone()));

        // Collect type hints from any dep this header references — by-value or
        // behind a pointer. Hints don't emit C code; they only feed the
        // `type_meta` map so function signatures use the correct renamed names
        // and type tags. Without entries for pointer-only deps, a typedef
        // referenced behind a pointer would default to `struct Aliased` rather
        // than the bare typedef name.
        let mut type_hints: Vec<CTypeDefinition> = Vec::new();
        for dep_pkg in by_value_from.iter().chain(pointer_only_from.iter()) {
            if let Some(dep_defs) = partitioned.per_crate.get(*dep_pkg) {
                type_hints.extend(dep_defs.iter().cloned());
            }
        }

        result.insert(
            pkg_id.clone(),
            HeaderDeps {
                includes,
                forward_decls,
                type_hints,
            },
        );
    }

    result
}

/// Pre-computed mapping from `PackageId` to header filename (without extension).
///
/// Uses each package's library target name as the base filename. When multiple
/// packages share the same library name, the version is appended for
/// disambiguation (e.g. `libc_0_2`), using the minimum number of semver
/// components needed.
///
/// Per-package `header_name` renames from config are applied up front and
/// bypass version disambiguation entirely.
pub struct HeaderFilenames {
    names: HashMap<PackageId, String>,
}

/// Compute the default header base name for a package: library target name if
/// present, otherwise the package name with dashes replaced by underscores.
pub fn default_header_base_name(graph: &PackageGraph, pkg_id: &PackageId) -> Option<String> {
    let meta = graph.metadata(pkg_id).ok()?;
    let name = meta
        .build_targets()
        .find(|t| matches!(t.id(), BuildTargetId::Library))
        .map(|t| t.name().to_owned())
        .unwrap_or_else(|| meta.name().replace('-', "_"));
    Some(name)
}

impl HeaderFilenames {
    /// Build the filename map for the given set of package IDs, applying any
    /// rename overrides.
    ///
    /// Returns an error if two packages end up with the same base name (either
    /// via colliding renames, or a rename colliding with another package's
    /// default name that cannot be disambiguated).
    pub fn new(
        package_ids: &[&PackageId],
        graph: &PackageGraph,
        renames: &HashMap<PackageId, String>,
    ) -> Result<Self, String> {
        // Step 1: resolve each package ID to its base name. Renamed packages
        // use their override directly and skip version disambiguation; the
        // rest use the library target name (falling back to the package name
        // with dashes replaced by underscores).
        let mut renamed: HashMap<PackageId, String> = HashMap::new();
        let mut defaults: Vec<(&PackageId, String, Version)> = Vec::new();
        for &id in package_ids {
            if let Some(rename) = renames.get(id) {
                renamed.insert(id.clone(), rename.clone());
                continue;
            }
            let Some(meta) = graph.metadata(id).ok() else {
                continue;
            };
            let lib_name = meta
                .build_targets()
                .find(|t| matches!(t.id(), BuildTargetId::Library))
                .map(|t| t.name().to_owned())
                .unwrap_or_else(|| meta.name().replace('-', "_"));
            defaults.push((id, lib_name, meta.version().clone()));
        }

        // Step 2: find duplicates among defaults and disambiguate with version.
        let mut name_counts: HashMap<&str, Vec<usize>> = HashMap::new();
        for (i, (_, name, _)) in defaults.iter().enumerate() {
            name_counts.entry(name.as_str()).or_default().push(i);
        }

        let mut names = HashMap::new();
        for indices in name_counts.values() {
            if indices.len() == 1 {
                // Unique name — no disambiguation needed.
                let i = indices[0];
                let (id, ref name, _) = defaults[i];
                names.insert(id.clone(), name.clone());
            } else {
                // Multiple packages with the same lib name — disambiguate.
                let entries: Vec<_> = indices
                    .iter()
                    .map(|&i| {
                        let (id, ref name, ref version) = defaults[i];
                        (id, name.clone(), version.clone())
                    })
                    .collect();

                // Try major only, then major.minor, then full version.
                let major_unique = {
                    let majors: HashSet<_> = entries.iter().map(|(_, _, v)| v.major).collect();
                    majors.len() == entries.len()
                };

                for (id, name, version) in &entries {
                    let suffix = if major_unique {
                        format!("_{}", version.major)
                    } else {
                        let minor_unique = {
                            let minor_keys: HashSet<_> =
                                entries.iter().map(|(_, _, v)| (v.major, v.minor)).collect();
                            minor_keys.len() == entries.len()
                        };
                        if minor_unique {
                            format!("_{}_{}", version.major, version.minor)
                        } else {
                            format!("_{}_{}_{}", version.major, version.minor, version.patch)
                        }
                    };
                    names.insert((*id).clone(), format!("{name}{suffix}"));
                }
            }
        }

        // Step 3: merge renames in. Renames never collide with each other
        // (validated upstream), but a rename may collide with a default name;
        // detect and report that here.
        for (id, rename) in renamed {
            if let Some((other_id, _)) = names.iter().find(|(_, n)| **n == rename) {
                let (renamed_label, other_label) = (
                    graph
                        .metadata(&id)
                        .map(|m| format!("{}@{}", m.name(), m.version()))
                        .unwrap_or_else(|_| id.repr().to_owned()),
                    graph
                        .metadata(other_id)
                        .map(|m| format!("{}@{}", m.name(), m.version()))
                        .unwrap_or_else(|_| other_id.repr().to_owned()),
                );
                return Err(format!(
                    "`header_name = \"{rename}\"` on `{renamed_label}` collides with \
                     the default header name for `{other_label}`; rename one of them"
                ));
            }
            names.insert(id, rename);
        }

        Ok(Self { names })
    }

    /// Get the header filename (with extension) for a package.
    pub fn filename(&self, id: &PackageId, lang_extension: &str) -> String {
        let base = self
            .names
            .get(id)
            .expect("package not in HeaderFilenames map");
        format!("{base}.{lang_extension}")
    }

    /// Get the base name (without extension) for a package.
    pub fn base_name(&self, id: &PackageId) -> &str {
        self.names
            .get(id)
            .expect("package not in HeaderFilenames map")
    }
}

/// Map a [`CTypeKind`] to the kind that should be emitted when the type is
/// referenced only behind a pointer in the consumer header.
///
/// Compounds collapse to opaque struct/union forward declarations, with the
/// tag chosen to match what `codegen` would emit for the full definition (a
/// `repr(uN)` tagged union renders as a C `union`, so its forward decl must
/// say `union`, not `struct`). Typedefs and fieldless enums cannot be
/// forward-declared in standard C — they're returned unchanged so the full
/// definition is emitted inline.
fn forward_decl_kind(kind: &CTypeKind) -> CTypeKind {
    match kind {
        CTypeKind::Union(_) | CTypeKind::OpaqueUnion => CTypeKind::OpaqueUnion,
        CTypeKind::Struct(_) | CTypeKind::OpaqueStruct => CTypeKind::OpaqueStruct,
        CTypeKind::TaggedUnion(t) if t.repr.is_repr_c() => CTypeKind::OpaqueStruct,
        CTypeKind::TaggedUnion(_) => CTypeKind::OpaqueUnion,
        other => other.clone(),
    }
}

/// Collect the C type names of all types referenced by-value in a type definition's fields.
fn collect_by_value_type_names(def: &CTypeDefinition) -> Vec<String> {
    let mut names = Vec::new();
    match &def.kind {
        CTypeKind::Struct(s) => {
            for field in &s.fields {
                collect_by_value_names_from_type(&field.type_, &mut names);
            }
        }
        CTypeKind::Union(u) => {
            for field in &u.fields {
                collect_by_value_names_from_type(&field.type_, &mut names);
            }
        }
        CTypeKind::TaggedUnion(t) => {
            for variant in &t.variants {
                if let Some(ref body) = variant.body {
                    for field in &body.fields {
                        collect_by_value_names_from_type(&field.type_, &mut names);
                    }
                }
            }
        }
        CTypeKind::Typedef(td) => {
            collect_by_value_names_from_type(&td.inner, &mut names);
        }
        CTypeKind::FieldlessEnum(_) | CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion => {}
    }
    names
}

/// Collect C type names from a type tree, following by-value paths only.
fn collect_by_value_names_from_type(ty: &Type, names: &mut Vec<String>) {
    match ty {
        Type::Path(_) | Type::TypeAlias(_) => {
            names.push(c_type_name(ty));
        }
        Type::Array(a) => collect_by_value_names_from_type(&a.element_type, names),
        Type::Tuple(t) => {
            for elem in &t.elements {
                collect_by_value_names_from_type(elem, names);
            }
        }
        // Pointers/references are not by-value.
        _ => {}
    }
}

/// Collect C type names that appear behind a pointer/reference in a type definition.
fn collect_pointer_type_names(def: &CTypeDefinition) -> Vec<String> {
    let mut names = Vec::new();
    match &def.kind {
        CTypeKind::Struct(s) => {
            for field in &s.fields {
                collect_pointer_names_from_type(&field.type_, &mut names);
            }
        }
        CTypeKind::Union(u) => {
            for field in &u.fields {
                collect_pointer_names_from_type(&field.type_, &mut names);
            }
        }
        CTypeKind::TaggedUnion(t) => {
            for variant in &t.variants {
                if let Some(ref body) = variant.body {
                    for field in &body.fields {
                        collect_pointer_names_from_type(&field.type_, &mut names);
                    }
                }
            }
        }
        CTypeKind::Typedef(td) => {
            collect_pointer_names_from_type(&td.inner, &mut names);
        }
        CTypeKind::FieldlessEnum(_) | CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion => {}
    }
    names
}

/// Walk a type tree, collecting C names of types found behind pointers/references.
fn collect_pointer_names_from_type(ty: &Type, names: &mut Vec<String>) {
    match ty {
        Type::RawPointer(p) => {
            // The pointed-to type is used behind a pointer.
            collect_pointed_names(&p.inner, names);
        }
        Type::Reference(r) => {
            collect_pointed_names(&r.inner, names);
        }
        Type::Array(a) => collect_pointer_names_from_type(&a.element_type, names),
        _ => {}
    }
}

/// Collect C name of the type behind a pointer (first path type found).
fn collect_pointed_names(ty: &Type, names: &mut Vec<String>) {
    match ty {
        Type::Path(_) | Type::TypeAlias(_) => {
            names.push(c_type_name(ty));
        }
        Type::Array(a) => collect_pointed_names(&a.element_type, names),
        Type::RawPointer(p) => collect_pointed_names(&p.inner, names),
        Type::Reference(r) => collect_pointed_names(&r.inner, names),
        _ => {}
    }
}

/// Collect all referenced type names (both by-value and behind pointers).
fn collect_all_referenced_type_names(def: &CTypeDefinition) -> Vec<String> {
    let mut names = collect_by_value_type_names(def);
    names.extend(collect_pointer_type_names(def));
    names
}

/// Collect type names from function signatures and static types.
fn collect_fn_type_names(fns: &[FreeFunctionItem], statics: &[StaticItem]) -> Vec<String> {
    let mut names = Vec::new();
    for item in fns {
        let func = &item.function;
        for input in &func.header.inputs {
            collect_by_value_names_from_type(&input.type_, &mut names);
        }
        if let Some(ref output) = func.header.output {
            collect_by_value_names_from_type(output, &mut names);
        }
    }
    for s in statics {
        collect_by_value_names_from_type(&s.type_, &mut names);
    }
    names
}
