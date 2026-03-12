use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use rustdoc_ir::{FreeFunction, GenericArgument, PathType, Type};
use rustdoc_processor::CrateCollection;
use rustdoc_processor::indexing::{CrateIndexer, NoAnnotations};
use rustdoc_processor::queries::Crate;
use rustdoc_resolver::{resolve_free_function, resolve_type};
use rustdoc_types::{Abi, Attribute, AttributeRepr, ItemEnum, ReprKind, StructKind};

use crate::config::SortKey;
use crate::static_item::{StaticItem, resolve_static};

/// A user-defined type that needs a C declaration in the header.
pub struct CTypeDefinition {
    /// The C name for this type (last path segment from PathType::base_type).
    pub name: String,
    /// Whether this is an opaque forward declaration or a full struct definition.
    pub kind: CTypeKind,
    /// The rustdoc item ID, used for doc comment lookup at codegen time.
    pub rustdoc_id: Option<rustdoc_types::Id>,
}

/// The kind of C type definition to emit.
pub enum CTypeKind {
    /// Emit only a forward declaration (`struct Foo;` / `typedef struct Foo Foo;`).
    Opaque,
    /// Emit a full struct definition with fields.
    Struct(CStructDef),
}

/// A resolved `#[repr(C)]` struct with its fields.
pub struct CStructDef {
    /// The fields of the struct, in declaration order.
    pub fields: Vec<CStructField>,
}

/// A single field of a C struct.
pub struct CStructField {
    /// The C field name (Rust name for plain structs, `m0`/`m1`/... for tuple structs).
    pub name: String,
    /// The resolved type of this field.
    pub type_: Type,
}

/// Extern "C" function IDs and exported static IDs found in a crate.
pub struct ExternItems {
    pub fn_ids: Vec<rustdoc_types::Id>,
    pub static_ids: Vec<rustdoc_types::Id>,
}

/// Walk the crate's import index and collect extern "C" functions and exported statics.
pub fn find_extern_items(krate: &Crate, fn_sort_by: SortKey, static_sort_by: SortKey) -> ExternItems {
    let mut fn_ids = Vec::new();
    let mut static_ids = Vec::new();

    for id in krate.import_index.items.keys() {
        let Some(item) = krate.core.krate.index.get(id) else {
            continue;
        };
        match &item.inner {
            ItemEnum::Function(func) if matches!(func.header.abi, Abi::C { .. }) => {
                fn_ids.push(*id);
            }
            ItemEnum::Static(_) if has_export_attr(&item.attrs) => {
                static_ids.push(*id);
            }
            _ => {}
        }
    }

    sort_ids(&mut fn_ids, fn_sort_by, krate);
    sort_ids(&mut static_ids, static_sort_by, krate);

    ExternItems { fn_ids, static_ids }
}

/// Sort key: (line, column) from the item's span, falling back to name for items without spans.
fn span_sort_key(id: &rustdoc_types::Id, krate: &Crate) -> (usize, usize, String) {
    let Some(item) = krate.core.krate.index.get(id) else {
        return (usize::MAX, usize::MAX, String::new());
    };
    match item.span.as_ref() {
        Some(span) => (span.begin.0, span.begin.1, String::new()),
        None => (
            usize::MAX,
            usize::MAX,
            item.name.clone().unwrap_or_default(),
        ),
    }
}

/// Sort a list of item IDs according to the given [`SortKey`].
fn sort_ids(ids: &mut [rustdoc_types::Id], sort_by: SortKey, krate: &Crate) {
    match sort_by {
        SortKey::SourceOrder => ids.sort_by_cached_key(|id| span_sort_key(id, krate)),
        SortKey::Name => ids.sort_by_cached_key(|id| name_sort_key(id, krate)),
    }
}

/// Sort key: item name, alphabetically.
fn name_sort_key(id: &rustdoc_types::Id, krate: &Crate) -> String {
    krate
        .core
        .krate
        .index
        .get(id)
        .and_then(|item| item.name.clone())
        .unwrap_or_default()
}

/// Resolve each extern "C" function ID into the IR, validating types along the way.
pub fn resolve_functions(
    fn_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &CrateCollection<NoAnnotations>,
) -> anyhow::Result<Vec<FreeFunction>> {
    let mut resolved_fns = Vec::new();
    for id in fn_ids {
        let item = krate
            .core
            .krate
            .index
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Missing item for id {:?}", id))?;
        let free_fn = resolve_free_function(&item, krate, collection)
            .map_err(|e| anyhow::anyhow!("Failed to resolve function: {e}"))?;

        resolved_fns.push(free_fn);
    }
    Ok(resolved_fns)
}

/// Resolve each exported static ID into a [`StaticItem`].
pub fn resolve_statics(
    static_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &CrateCollection<NoAnnotations>,
) -> anyhow::Result<Vec<StaticItem>> {
    let mut resolved = Vec::new();
    for id in static_ids {
        let item = krate
            .core
            .krate
            .index
            .get(id)
            .ok_or_else(|| anyhow::anyhow!("Missing item for id {:?}", id))?;
        let static_item = resolve_static(&item, krate, collection)
            .map_err(|e| anyhow::anyhow!("Failed to resolve static: {e}"))?;
        resolved.push(static_item);
    }
    Ok(resolved)
}

/// Extract symbol names from function and static IDs.
pub fn collect_symbols(
    items: &ExternItems,
    krate: &Crate,
) -> BTreeSet<String> {
    let mut symbols = BTreeSet::new();
    for id in items.fn_ids.iter().chain(&items.static_ids) {
        let Some(item) = krate.core.krate.index.get(id) else {
            continue;
        };
        if let Some(name) = exported_symbol_name(&item) {
            symbols.insert(name.to_owned());
        }
    }
    symbols
}

/// Return the linker-visible symbol name for an item.
///
/// Priority: `#[export_name = "..."]` > `item.name` (for `#[no_mangle]`).
fn exported_symbol_name(item: &rustdoc_types::Item) -> Option<&str> {
    for attr in &item.attrs {
        if let Attribute::ExportName(name) = attr {
            return Some(name);
        }
    }
    item.name.as_deref()
}

/// Returns `true` if the item has `#[no_mangle]` or `#[export_name = "..."]`.
fn has_export_attr(attrs: &[Attribute]) -> bool {
    attrs
        .iter()
        .any(|a| matches!(a, Attribute::NoMangle | Attribute::ExportName(_)))
}

/// Walk all types in function signatures and static items, collecting unique
/// path types that need C declarations in the generated header.
///
/// Types used directly (not only behind pointers) and marked `#[repr(C)]` get
/// full struct definitions. All others get forward declarations.
pub fn collect_type_definitions<I: CrateIndexer>(
    functions: &[FreeFunction],
    statics: &[StaticItem],
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> anyhow::Result<Vec<CTypeDefinition>> {
    let mut seen: HashMap<PathType, bool> = HashMap::new();
    for func in functions {
        for input in &func.header.inputs {
            collect_paths_from_type(&input.type_, false, &mut seen);
        }
        if let Some(output) = &func.header.output {
            collect_paths_from_type(output, false, &mut seen);
        }
    }
    for s in statics {
        collect_paths_from_type(&s.type_, false, &mut seen);
    }

    // Resolve struct fields for directly-used types. This may discover new
    // transitive types that also need definitions.
    resolve_all_type_definitions(&mut seen, krate, collection)
}

/// Compute the cbindgen-style monomorphized C name for a type.
///
/// - Scalars use their Rust name (e.g. `"i32"`, `"bool"`).
/// - Path types use the last segment plus mangled generic arguments.
///   Generic arguments at the same level are separated by `__` (double underscore),
///   and a single `_` separates the base name from the first argument.
///   Lifetimes are ignored.
/// - References and raw pointers recurse into the inner type.
pub fn c_type_name(ty: &Type) -> String {
    match ty {
        Type::ScalarPrimitive(p) => p.as_str().to_owned(),
        Type::Path(p) => {
            let base = p.base_type.last().expect("empty path");
            let type_args: Vec<String> = p
                .generic_arguments
                .iter()
                .filter_map(|arg| match arg {
                    GenericArgument::TypeParameter(t) => Some(c_type_name(t)),
                    GenericArgument::Lifetime(_) => None,
                })
                .collect();
            if type_args.is_empty() {
                base.clone()
            } else {
                format!("{}_{}", base, type_args.join("__"))
            }
        }
        Type::Reference(r) => c_type_name(&r.inner),
        Type::RawPointer(r) => c_type_name(&r.inner),
        Type::Tuple(t) => {
            // Tuples shouldn't typically appear in monomorphized names,
            // but handle gracefully.
            let elems: Vec<String> = t.elements.iter().map(c_type_name).collect();
            elems.join("__")
        }
        Type::Slice(s) => c_type_name(&s.element_type),
        Type::Array(a) => c_type_name(&a.element_type),
        Type::Generic(g) => g.name.clone(),
    }
}

fn collect_paths_from_type(
    ty: &Type,
    behind_pointer: bool,
    seen: &mut HashMap<PathType, bool>,
) {
    match ty {
        Type::Path(p) => {
            let used_directly = !behind_pointer;
            let entry = seen.entry(p.clone()).or_insert(false);
            if used_directly {
                *entry = true;
            }
        }
        Type::Reference(r) => collect_paths_from_type(&r.inner, true, seen),
        Type::RawPointer(r) => collect_paths_from_type(&r.inner, true, seen),
        Type::Tuple(t) => {
            for element in &t.elements {
                collect_paths_from_type(element, behind_pointer, seen);
            }
        }
        Type::Slice(s) => collect_paths_from_type(&s.element_type, behind_pointer, seen),
        Type::Array(a) => collect_paths_from_type(&a.element_type, behind_pointer, seen),
        Type::ScalarPrimitive(_) | Type::Generic(_) => {}
    }
}

/// Zero-sized types that should be skipped when emitting struct fields.
fn is_zst_type(ty: &rustdoc_types::Type) -> bool {
    match ty {
        // `()` — empty tuple
        rustdoc_types::Type::Tuple(elems) if elems.is_empty() => true,
        // `PhantomData<T>` and `PhantomPinned`
        rustdoc_types::Type::ResolvedPath(path) => {
            let p = &path.path;
            p.ends_with("PhantomData") || p.ends_with("PhantomPinned")
        }
        _ => false,
    }
}

/// Resolve all collected types into `CTypeDefinition`s, iterating to a fixed
/// point to discover transitive field types.
fn resolve_all_type_definitions<I: CrateIndexer>(
    seen: &mut HashMap<PathType, bool>,
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> anyhow::Result<Vec<CTypeDefinition>> {
    // Phase 1: fixed-point loop over directly-used types.
    // By resolving all direct uses first, we ensure that a type initially seen
    // behind a pointer gets upgraded to direct when a struct field references it
    // by value — before we emit any opaques.
    let mut resolved: HashMap<PathType, CTypeDefinition> = HashMap::new();
    loop {
        let direct: Vec<PathType> = seen
            .iter()
            .filter(|(pt, used_directly)| **used_directly && !resolved.contains_key(*pt))
            .map(|(pt, _)| pt.clone())
            .collect();

        if direct.is_empty() {
            break;
        }

        for path_type in direct {
            let name = c_type_name(&Type::Path(path_type.clone()));
            let kind = resolve_struct_kind(&name, &path_type, krate, collection)?;

            // If we resolved a full struct, discover transitive field types.
            if let CTypeKind::Struct(ref def) = kind {
                for field in &def.fields {
                    collect_paths_from_type(&field.type_, false, seen);
                }
            }

            resolved.insert(
                path_type.clone(),
                CTypeDefinition {
                    name,
                    kind,
                    rustdoc_id: path_type.rustdoc_id,
                },
            );
        }
    }

    // Phase 2: emit remaining pointer-only types as opaque forward declarations.
    let opaque: Vec<PathType> = seen
        .keys()
        .filter(|pt| !resolved.contains_key(*pt))
        .cloned()
        .collect();
    for path_type in opaque {
        let name = c_type_name(&Type::Path(path_type.clone()));
        let rustdoc_id = path_type.rustdoc_id;
        resolved.insert(
            path_type,
            CTypeDefinition {
                name,
                kind: CTypeKind::Opaque,
                rustdoc_id,
            },
        );
    }

    let mut defs: Vec<CTypeDefinition> = resolved.into_values().collect();
    defs.sort_by(|a, b| a.name.cmp(&b.name));
    Ok(defs)
}

/// Attempt to resolve a directly-used type into a full struct definition.
///
/// Returns `CTypeKind::Opaque` (with a warning on stderr) if the type cannot
/// be fully defined — e.g. missing `rustdoc_id`, not `#[repr(C)]`, has
/// stripped fields, or is not a struct.
fn resolve_struct_kind<I: CrateIndexer>(
    name: &str,
    path_type: &PathType,
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
    let Some(id) = &path_type.rustdoc_id else {
        eprintln!("warning: type `{name}` has no rustdoc ID; emitting forward declaration");
        return Ok(CTypeKind::Opaque);
    };

    let Some(item) = krate.core.krate.index.get(id) else {
        eprintln!("warning: type `{name}` has no rustdoc ID; emitting forward declaration");
        return Ok(CTypeKind::Opaque);
    };

    let ItemEnum::Struct(struct_def) = &item.inner else {
        eprintln!("warning: type `{name}` is not a struct; emitting forward declaration");
        return Ok(CTypeKind::Opaque);
    };

    // Check for #[repr(C)].
    let is_repr_c = item.attrs.iter().any(|attr| {
        matches!(
            attr,
            Attribute::Repr(AttributeRepr {
                kind: ReprKind::C,
                ..
            })
        )
    });
    if !is_repr_c {
        eprintln!(
            "warning: type `{name}` is not #[repr(C)]; emitting opaque forward declaration"
        );
        return Ok(CTypeKind::Opaque);
    }

    // Check for unbound generic type parameters — treat as opaque.
    let has_type_params = struct_def.generics.params.iter().any(|p| {
        matches!(
            p.kind,
            rustdoc_types::GenericParamDefKind::Type { .. }
        )
    });
    if has_type_params {
        eprintln!(
            "warning: type `{name}` has generic type parameters; emitting forward declaration"
        );
        return Ok(CTypeKind::Opaque);
    }

    match &struct_def.kind {
        StructKind::Plain {
            fields,
            has_stripped_fields,
        } => {
            if *has_stripped_fields {
                eprintln!(
                    "warning: type `{name}` has private fields; emitting opaque forward declaration"
                );
                return Ok(CTypeKind::Opaque);
            }
            let c_fields =
                resolve_plain_fields(fields, krate, collection)?;
            Ok(CTypeKind::Struct(CStructDef { fields: c_fields }))
        }
        StructKind::Tuple(fields) => {
            let c_fields =
                resolve_tuple_fields(fields, krate, collection)?;
            Ok(CTypeKind::Struct(CStructDef { fields: c_fields }))
        }
        StructKind::Unit => Ok(CTypeKind::Struct(CStructDef {
            fields: Vec::new(),
        })),
    }
}

/// Resolve named struct fields into C struct fields.
fn resolve_plain_fields<I: CrateIndexer>(
    field_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> anyhow::Result<Vec<CStructField>> {
    let mut c_fields = Vec::new();
    for field_id in field_ids {
        let field_item = krate
            .core
            .krate
            .index
            .get(field_id)
            .ok_or_else(|| anyhow::anyhow!("Missing field item for id {:?}", field_id))?;
        let ItemEnum::StructField(ref raw_type) = field_item.inner else {
            anyhow::bail!("Expected StructField for id {:?}", field_id);
        };

        // Skip ZST fields (PhantomData, PhantomPinned, ()).
        if is_zst_type(raw_type) {
            continue;
        }

        let field_name = field_item
            .name
            .clone()
            .unwrap_or_else(|| "<unnamed>".to_string());
        let resolved = resolve_type(
            raw_type,
            &krate.core.package_id,
            collection,
            &Default::default(),
        )
        .map_err(|e| anyhow::anyhow!("Failed to resolve field `{field_name}`: {}", Arc::new(e)))?;

        c_fields.push(CStructField {
            name: field_name,
            type_: resolved,
        });
    }
    Ok(c_fields)
}

/// Resolve tuple struct fields into C struct fields named `m0`, `m1`, etc.
fn resolve_tuple_fields<I: CrateIndexer>(
    fields: &[Option<rustdoc_types::Id>],
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> anyhow::Result<Vec<CStructField>> {
    let mut c_fields = Vec::new();
    let mut index = 0;
    for slot in fields {
        let Some(field_id) = slot else {
            // Private/hidden field — we can't emit the full struct.
            anyhow::bail!("Tuple struct has private fields");
        };
        let field_item = krate
            .core
            .krate
            .index
            .get(field_id)
            .ok_or_else(|| anyhow::anyhow!("Missing tuple field item for id {:?}", field_id))?;
        let ItemEnum::StructField(ref raw_type) = field_item.inner else {
            anyhow::bail!("Expected StructField for tuple field id {:?}", field_id);
        };

        // Skip ZST fields.
        if is_zst_type(raw_type) {
            continue;
        }

        let resolved = resolve_type(
            raw_type,
            &krate.core.package_id,
            collection,
            &Default::default(),
        )
        .map_err(|e| {
            anyhow::anyhow!("Failed to resolve tuple field m{index}: {}", Arc::new(e))
        })?;

        c_fields.push(CStructField {
            name: format!("m{index}"),
            type_: resolved,
        });
        index += 1;
    }
    Ok(c_fields)
}
