use std::collections::{BTreeSet, HashMap};
use std::sync::Arc;

use rustdoc_ir::{FreeFunction, GenericArgument, PathType, ScalarPrimitive, Type};
use rustdoc_processor::{CORE_PACKAGE_ID_REPR, CrateCollection, STD_PACKAGE_ID_REPR};
use rustdoc_processor::indexing::{CrateIndexer, NoAnnotations};
use rustdoc_processor::queries::Crate;
use rustdoc_resolver::{GenericBindings, resolve_free_function, resolve_type};
use rustdoc_types::{Abi, Attribute, AttributeRepr, ItemEnum, ReprKind, StructKind, VariantKind};

use crate::config::SortKey;
use crate::constant_item::{ConstantItem, resolve_constant};
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
    /// Emit a C-like enum (no data variants).
    FieldlessEnum(CFieldlessEnumDef),
    /// Emit a tagged union (enum with data variants).
    TaggedUnion(CTaggedUnionDef),
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

/// A C-like enum (all variants are fieldless).
pub struct CFieldlessEnumDef {
    pub repr: CEnumRepr,
    pub variants: Vec<CEnumVariant>,
}

/// A single variant of a fieldless enum.
pub struct CEnumVariant {
    pub name: String,
    pub discriminant: Option<String>,
}

/// Primitive integer types valid in `#[repr(...)]` on Rust enums.
/// See: https://doc.rust-lang.org/reference/type-layout.html#r-layout.repr.primitive
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReprIntType {
    U8,
    U16,
    U32,
    U64,
    U128,
    Usize,
    I8,
    I16,
    I32,
    I64,
    I128,
    Isize,
}

impl ReprIntType {
    /// Parse from the string found in `AttributeRepr::int`.
    pub fn parse(s: &str) -> anyhow::Result<Self> {
        match s {
            "u8" => Ok(Self::U8),
            "u16" => Ok(Self::U16),
            "u32" => Ok(Self::U32),
            "u64" => Ok(Self::U64),
            "u128" => Ok(Self::U128),
            "usize" => Ok(Self::Usize),
            "i8" => Ok(Self::I8),
            "i16" => Ok(Self::I16),
            "i32" => Ok(Self::I32),
            "i64" => Ok(Self::I64),
            "i128" => Ok(Self::I128),
            "isize" => Ok(Self::Isize),
            other => anyhow::bail!("unknown repr integer type `{other}`"),
        }
    }

    /// Convert to the corresponding `ScalarPrimitive` for reuse in codegen.
    pub fn to_scalar_primitive(self) -> ScalarPrimitive {
        match self {
            Self::U8 => ScalarPrimitive::U8,
            Self::U16 => ScalarPrimitive::U16,
            Self::U32 => ScalarPrimitive::U32,
            Self::U64 => ScalarPrimitive::U64,
            Self::U128 => ScalarPrimitive::U128,
            Self::Usize => ScalarPrimitive::Usize,
            Self::I8 => ScalarPrimitive::I8,
            Self::I16 => ScalarPrimitive::I16,
            Self::I32 => ScalarPrimitive::I32,
            Self::I64 => ScalarPrimitive::I64,
            Self::I128 => ScalarPrimitive::I128,
            Self::Isize => ScalarPrimitive::Isize,
        }
    }
}

/// How the enum's discriminant is represented in C.
pub enum CEnumRepr {
    /// `#[repr(C)]` only — use a plain C enum.
    C,
    /// `#[repr(uN)]` or `#[repr(C, uN)]` — emit enum constants + typedef to int type.
    Int { is_repr_c: bool, int_type: ReprIntType },
}

impl CEnumRepr {
    pub fn is_repr_c(&self) -> bool {
        match self {
            CEnumRepr::C => true,
            CEnumRepr::Int { is_repr_c, .. } => *is_repr_c,
        }
    }
}

/// A tagged union (enum with data variants).
pub struct CTaggedUnionDef {
    pub repr: CEnumRepr,
    /// When true, variant names in the tag enum are prefixed with the enum name.
    pub prefix_with_name: bool,
    pub variants: Vec<CTaggedVariant>,
}

/// A single variant of a tagged union.
pub struct CTaggedVariant {
    pub name: String,
    pub body: Option<CTaggedVariantBody>,
}

/// The body (fields) of a tagged union variant.
pub struct CTaggedVariantBody {
    pub fields: Vec<CStructField>,
}

/// Extern "C" function IDs, exported static IDs, and constant IDs found in a crate.
pub struct ExternItems {
    pub fn_ids: Vec<rustdoc_types::Id>,
    pub static_ids: Vec<rustdoc_types::Id>,
    pub constant_ids: Vec<rustdoc_types::Id>,
}

/// Walk the crate's import index and collect extern "C" functions, exported statics,
/// and public constants.
pub fn find_extern_items(krate: &Crate) -> ExternItems {
    let mut fn_ids = Vec::new();
    let mut static_ids = Vec::new();
    let mut constant_ids = Vec::new();

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
            ItemEnum::Constant { .. } => {
                constant_ids.push(*id);
            }
            _ => {}
        }
    }

    ExternItems {
        fn_ids,
        static_ids,
        constant_ids,
    }
}

/// Sort key: (line, column) from the item's span.
fn span_sort_key(id: &rustdoc_types::Id, krate: &Crate) -> (usize, usize) {
    let Some(item) = krate.core.krate.index.get(id) else {
        return (usize::MAX, usize::MAX);
    };
    match item.span.as_ref() {
        Some(span) => (span.begin.0, span.begin.1),
        None => (usize::MAX, usize::MAX),
    }
}

/// Trait for items that can be sorted by their rustdoc ID.
pub trait HasRustdocId {
    fn rustdoc_id(&self) -> Option<&rustdoc_types::Id>;
    fn fallback_name(&self) -> String;
}

impl HasRustdocId for rustdoc_types::Id {
    fn rustdoc_id(&self) -> Option<&rustdoc_types::Id> {
        Some(self)
    }

    fn fallback_name(&self) -> String {
        String::new()
    }
}

impl HasRustdocId for CTypeDefinition {
    fn rustdoc_id(&self) -> Option<&rustdoc_types::Id> {
        self.rustdoc_id.as_ref()
    }

    fn fallback_name(&self) -> String {
        self.name.clone()
    }
}

/// Sort a slice of items according to the given [`SortKey`], using each
/// item's rustdoc ID to look up source position or name.
pub fn sort_by_key<T: HasRustdocId>(items: &mut [T], sort_by: SortKey, krate: &Crate) {
    match sort_by {
        SortKey::SourceOrder => items.sort_by_cached_key(|item| {
            let (line, col) = match item.rustdoc_id() {
                Some(id) => span_sort_key(id, krate),
                None => (usize::MAX, usize::MAX),
            };
            // Tiebreak on the item's own name so that multiple
            // monomorphisations of the same generic (same span) sort
            // deterministically.
            (line, col, item.fallback_name())
        }),
        SortKey::Name => items.sort_by_cached_key(|item| {
            match item.rustdoc_id() {
                Some(id) => name_sort_key(id, krate),
                None => item.fallback_name(),
            }
        }),
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

/// Resolve each constant ID into a [`ConstantItem`], skipping unsupported types.
pub fn resolve_constants(
    constant_ids: &[rustdoc_types::Id],
    krate: &Crate,
    collection: &CrateCollection<NoAnnotations>,
) -> Vec<ConstantItem> {
    let mut resolved = Vec::new();
    for id in constant_ids {
        let Some(item) = krate.core.krate.index.get(id) else {
            continue;
        };
        if let Some(constant) = resolve_constant(&item, krate, collection) {
            resolved.push(constant);
        }
    }
    resolved
}

/// Extract symbol names from function and static IDs.
pub fn collect_symbols(items: &ExternItems, krate: &Crate) -> BTreeSet<String> {
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
        Type::FunctionPointer(_) => {
            unreachable!("unsupported type in C type name: {ty:?}")
        }
    }
}

fn collect_paths_from_type(ty: &Type, behind_pointer: bool, seen: &mut HashMap<PathType, bool>) {
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
        Type::ScalarPrimitive(_) | Type::Generic(_) | Type::FunctionPointer(_) => {}
    }
}

/// Zero-sized types that should be skipped when emitting struct fields.
fn is_zst_type(ty: &Type) -> bool {
    match ty {
        // `()` — empty tuple
        Type::Tuple(t) if t.elements.is_empty() => true,
        // `PhantomData<T>` and `PhantomPinned`
        Type::Path(PathType {
            package_id,
            base_type,
            ..
        }) => {
            let pkg = package_id.repr();
            if pkg != CORE_PACKAGE_ID_REPR && pkg != STD_PACKAGE_ID_REPR {
                return false;
            }
            matches!(
                base_type.iter().map(String::as_str).collect::<Vec<_>>().as_slice(),
                ["core" | "std", "marker", "PhantomData" | "PhantomPinned"]
            )
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
            let kind = resolve_type_kind(&name, &path_type, krate, collection)?;

            // Discover transitive field types from full definitions.
            match &kind {
                CTypeKind::Struct(def) => {
                    for field in &def.fields {
                        collect_paths_from_type(&field.type_, false, seen);
                    }
                }
                CTypeKind::TaggedUnion(def) => {
                    for variant in &def.variants {
                        if let Some(ref body) = variant.body {
                            for field in &body.fields {
                                collect_paths_from_type(&field.type_, false, seen);
                            }
                        }
                    }
                }
                CTypeKind::Opaque | CTypeKind::FieldlessEnum(_) => {}
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

    let defs: Vec<CTypeDefinition> = resolved.into_values().collect();
    Ok(defs)
}

/// Attempt to resolve a directly-used type into a full definition.
///
/// Returns `CTypeKind::Opaque` (with a warning on stderr) if the type cannot
/// be fully defined — e.g. missing `rustdoc_id`, not `#[repr(C)]`, has
/// stripped fields, etc.
fn resolve_type_kind<I: CrateIndexer>(
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

    match &item.inner {
        ItemEnum::Struct(struct_def) => {
            resolve_struct_kind(name, struct_def, &item.attrs, path_type, krate, collection)
        }
        ItemEnum::Enum(enum_def) => {
            resolve_enum_kind(name, enum_def, &item.attrs, path_type, krate, collection)
        }
        _ => {
            eprintln!(
                "warning: type `{name}` is not a struct or enum; emitting forward declaration"
            );
            Ok(CTypeKind::Opaque)
        }
    }
}

/// Resolve a struct into a `CTypeKind`.
fn resolve_struct_kind<I: CrateIndexer>(
    name: &str,
    struct_def: &rustdoc_types::Struct,
    attrs: &[Attribute],
    path_type: &PathType,
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
    // Check for #[repr(C)].
    let is_repr_c = attrs.iter().any(|attr| {
        matches!(
            attr,
            Attribute::Repr(AttributeRepr {
                kind: ReprKind::C,
                ..
            })
        )
    });
    if !is_repr_c {
        eprintln!("warning: type `{name}` is not #[repr(C)]; emitting opaque forward declaration");
        return Ok(CTypeKind::Opaque);
    }

    // Build generic substitution bindings when concrete arguments are provided.
    let mut generic_bindings = if has_type_params(&struct_def.generics) {
        match build_generic_bindings(&struct_def.generics, &path_type.generic_arguments) {
            Some(bindings) => bindings,
            None => {
                eprintln!(
                    "warning: type `{name}` has generic type parameters; emitting forward declaration"
                );
                return Ok(CTypeKind::Opaque);
            }
        }
    } else {
        GenericBindings::default()
    };

    // Bind `Self` to the containing type so that `*const Self` / `*mut Self`
    // fields resolve to the correct concrete type.
    generic_bindings
        .types
        .insert("Self".into(), Type::Path(path_type.clone()));

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
            let c_fields = resolve_plain_fields(fields, &generic_bindings, krate, collection)?;
            Ok(CTypeKind::Struct(CStructDef { fields: c_fields }))
        }
        StructKind::Tuple(fields) => {
            let c_fields = resolve_tuple_fields(fields, &generic_bindings, krate, collection)?;
            Ok(CTypeKind::Struct(CStructDef { fields: c_fields }))
        }
        StructKind::Unit => Ok(CTypeKind::Struct(CStructDef { fields: Vec::new() })),
    }
}

/// Returns true if the generics contain unbound type parameters.
fn has_type_params(generics: &rustdoc_types::Generics) -> bool {
    generics
        .params
        .iter()
        .any(|p| matches!(p.kind, rustdoc_types::GenericParamDefKind::Type { .. }))
}

/// Build a [`GenericBindings`] map pairing each type parameter in `generics`
/// with the corresponding concrete type from `generic_args`.
///
/// Returns `None` if the struct has type parameters but no (or insufficient)
/// concrete arguments were provided — the caller should treat the type as opaque.
fn build_generic_bindings(
    generics: &rustdoc_types::Generics,
    generic_args: &[GenericArgument],
) -> Option<GenericBindings> {
    let type_param_names: Vec<&str> = generics
        .params
        .iter()
        .filter_map(|p| match &p.kind {
            rustdoc_types::GenericParamDefKind::Type { .. } => Some(p.name.as_str()),
            _ => None,
        })
        .collect();

    let type_args: Vec<&Type> = generic_args
        .iter()
        .filter_map(|arg| match arg {
            GenericArgument::TypeParameter(t) => Some(t),
            GenericArgument::Lifetime(_) => None,
        })
        .collect();

    if type_param_names.is_empty() {
        return Some(GenericBindings::default());
    }
    if type_args.len() != type_param_names.len() {
        return None;
    }

    let mut bindings = GenericBindings::default();
    for (name, ty) in type_param_names.into_iter().zip(type_args) {
        bindings.types.insert(name.to_owned(), ty.clone());
    }
    Some(bindings)
}

/// Extract a `CEnumRepr` from the item's attributes.
/// Returns `None` if the enum has no valid C-compatible repr.
fn extract_enum_repr(attrs: &[Attribute]) -> anyhow::Result<Option<CEnumRepr>> {
    for attr in attrs {
        if let Attribute::Repr(repr) = attr {
            match (&repr.kind, &repr.int) {
                (ReprKind::C, None) => return Ok(Some(CEnumRepr::C)),
                (ReprKind::C, Some(int_str)) => {
                    let int_type = ReprIntType::parse(int_str)?;
                    return Ok(Some(CEnumRepr::Int {
                        is_repr_c: true,
                        int_type,
                    }));
                }
                (ReprKind::Rust, Some(int_str)) => {
                    let int_type = ReprIntType::parse(int_str)?;
                    return Ok(Some(CEnumRepr::Int {
                        is_repr_c: false,
                        int_type,
                    }));
                }
                _ => {}
            }
        }
    }
    Ok(None)
}

/// Resolve an enum into a `CTypeKind`.
fn resolve_enum_kind<I: CrateIndexer>(
    name: &str,
    enum_def: &rustdoc_types::Enum,
    attrs: &[Attribute],
    path_type: &PathType,
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
    let Some(repr) = extract_enum_repr(attrs)? else {
        eprintln!(
            "warning: enum `{name}` has no C-compatible repr; emitting opaque forward declaration"
        );
        return Ok(CTypeKind::Opaque);
    };

    let mut generic_bindings = if has_type_params(&enum_def.generics) {
        match build_generic_bindings(&enum_def.generics, &path_type.generic_arguments) {
            Some(bindings) => bindings,
            None => {
                eprintln!(
                    "warning: enum `{name}` has generic type parameters; emitting forward declaration"
                );
                return Ok(CTypeKind::Opaque);
            }
        }
    } else {
        GenericBindings::default()
    };

    // Bind `Self` to the containing type so that `*const Self` / `*mut Self`
    // fields resolve to the correct concrete type.
    generic_bindings
        .types
        .insert("Self".into(), Type::Path(path_type.clone()));

    if enum_def.has_stripped_variants {
        eprintln!("warning: enum `{name}` has stripped variants; emitting forward declaration");
        return Ok(CTypeKind::Opaque);
    }

    // Classify: all variants plain → fieldless, otherwise → tagged union.
    let mut all_plain = true;
    for variant_id in &enum_def.variants {
        let Some(variant_item) = krate.core.krate.index.get(variant_id) else {
            eprintln!("warning: enum `{name}` has missing variant; emitting forward declaration");
            return Ok(CTypeKind::Opaque);
        };
        let ItemEnum::Variant(variant) = &variant_item.inner else {
            anyhow::bail!(
                "Expected Variant for enum `{name}` variant id {:?}",
                variant_id
            );
        };
        if !matches!(variant.kind, VariantKind::Plain) {
            all_plain = false;
            break;
        }
    }

    if all_plain {
        resolve_fieldless_enum(name, enum_def, repr, krate)
    } else {
        resolve_tagged_union(name, enum_def, repr, &generic_bindings, krate, collection)
    }
}

/// Resolve a fieldless enum.
fn resolve_fieldless_enum(
    name: &str,
    enum_def: &rustdoc_types::Enum,
    repr: CEnumRepr,
    krate: &Crate,
) -> anyhow::Result<CTypeKind> {
    let mut variants = Vec::new();
    for variant_id in &enum_def.variants {
        let variant_item = krate
            .core
            .krate
            .index
            .get(variant_id)
            .ok_or_else(|| anyhow::anyhow!("Missing variant for enum `{name}`"))?;
        let ItemEnum::Variant(variant) = &variant_item.inner else {
            anyhow::bail!("Expected Variant for enum `{name}`");
        };
        let variant_name = variant_item.name.clone().unwrap_or_default();
        let discriminant = variant.discriminant.as_ref().map(|d| d.expr.clone());
        variants.push(CEnumVariant {
            name: variant_name,
            discriminant,
        });
    }
    Ok(CTypeKind::FieldlessEnum(CFieldlessEnumDef {
        repr,
        variants,
    }))
}

/// Resolve a tagged union (enum with data variants).
fn resolve_tagged_union<I: CrateIndexer>(
    name: &str,
    enum_def: &rustdoc_types::Enum,
    repr: CEnumRepr,
    generic_bindings: &GenericBindings,
    krate: &Crate,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
    // repr(C) or repr(C, uN) → prefix variant names with enum name.
    let prefix_with_name = repr.is_repr_c();

    let mut variants = Vec::new();
    for variant_id in &enum_def.variants {
        let variant_item = krate
            .core
            .krate
            .index
            .get(variant_id)
            .ok_or_else(|| anyhow::anyhow!("Missing variant for enum `{name}`"))?;
        let ItemEnum::Variant(variant) = &variant_item.inner else {
            anyhow::bail!("Expected Variant for enum `{name}`");
        };
        let variant_name = variant_item.name.clone().unwrap_or_default();

        let body = match &variant.kind {
            VariantKind::Plain => None,
            VariantKind::Tuple(fields) => {
                let c_fields = resolve_tuple_fields(fields, generic_bindings, krate, collection)?;
                if c_fields.is_empty() {
                    None
                } else {
                    Some(CTaggedVariantBody { fields: c_fields })
                }
            }
            VariantKind::Struct {
                fields,
                has_stripped_fields,
            } => {
                if *has_stripped_fields {
                    eprintln!(
                        "warning: enum `{name}` variant `{variant_name}` has stripped fields; emitting forward declaration"
                    );
                    return Ok(CTypeKind::Opaque);
                }
                let c_fields = resolve_plain_fields(fields, generic_bindings, krate, collection)?;
                if c_fields.is_empty() {
                    None
                } else {
                    Some(CTaggedVariantBody { fields: c_fields })
                }
            }
        };

        variants.push(CTaggedVariant {
            name: variant_name,
            body,
        });
    }

    Ok(CTypeKind::TaggedUnion(CTaggedUnionDef {
        repr,
        prefix_with_name,
        variants,
    }))
}

/// Resolve named struct fields into C struct fields.
fn resolve_plain_fields<I: CrateIndexer>(
    field_ids: &[rustdoc_types::Id],
    generic_bindings: &GenericBindings,
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

        let field_name = field_item
            .name
            .clone()
            .unwrap_or_else(|| "<unnamed>".to_string());
        let resolved = resolve_type(
            raw_type,
            &krate.core.package_id,
            collection,
            generic_bindings,
        )
        .map_err(|e| anyhow::anyhow!("Failed to resolve field `{field_name}`: {}", Arc::new(e)))?;

        // Skip ZST fields (PhantomData, PhantomPinned, ()).
        if is_zst_type(&resolved) {
            continue;
        }

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
    generic_bindings: &GenericBindings,
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

        let resolved = resolve_type(
            raw_type,
            &krate.core.package_id,
            collection,
            generic_bindings,
        )
        .map_err(|e| anyhow::anyhow!("Failed to resolve tuple field m{index}: {}", Arc::new(e)))?;

        // Skip ZST fields.
        if is_zst_type(&resolved) {
            continue;
        }

        c_fields.push(CStructField {
            name: format!("m{index}"),
            type_: resolved,
        });
        index += 1;
    }
    Ok(c_fields)
}
