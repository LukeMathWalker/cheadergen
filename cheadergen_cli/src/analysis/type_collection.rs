use std::collections::HashMap;
use std::fmt;

use guppy::PackageId;
use rustdoc_ir::{GenericArgument, PathType, ScalarPrimitive, Type};
use rustdoc_processor::{CORE_PACKAGE_ID_REPR, GlobalItemId, STD_PACKAGE_ID_REPR};

use super::type_resolution::resolve_type_kind;
use crate::Collection;
use crate::analysis::CCanonicalType;
use crate::analysis::exported_via_annotations;
use crate::analysis::extern_items::ExternItems;
use crate::cli::generate::PackageTypeOverrides;
use crate::diagnostic::DiagnosticSink;
use crate::indexing::ExportMode;

/// C and C++ reserved keywords that cannot be used as identifiers.
///
/// Lowercasing Rust variant names (e.g. `Continue` → `continue`) can produce
/// keywords. [`CIdentifier`] appends `_` to avoid collisions.
const C_KEYWORDS: &[&str] = &[
    "alignas",
    "alignof",
    "auto",
    "bool",
    "break",
    "case",
    "char",
    "const",
    "constexpr",
    "continue",
    "default",
    "do",
    "double",
    "else",
    "enum",
    "extern",
    "false",
    "float",
    "for",
    "goto",
    "if",
    "inline",
    "int",
    "long",
    "nullptr",
    "register",
    "restrict",
    "return",
    "short",
    "signed",
    "sizeof",
    "static",
    "static_assert",
    "struct",
    "switch",
    "thread_local",
    "true",
    "typedef",
    "typeof",
    "typeof_unqual",
    "union",
    "unsigned",
    "void",
    "volatile",
    "while",
    // C++ additional keywords
    "and",
    "and_eq",
    "asm",
    "bitand",
    "bitor",
    "catch",
    "class",
    "compl",
    "concept",
    "const_cast",
    "consteval",
    "constinit",
    "co_await",
    "co_return",
    "co_yield",
    "decltype",
    "delete",
    "dynamic_cast",
    "explicit",
    "export",
    "friend",
    "mutable",
    "namespace",
    "new",
    "noexcept",
    "not",
    "not_eq",
    "operator",
    "or",
    "or_eq",
    "private",
    "protected",
    "public",
    "reinterpret_cast",
    "requires",
    "static_cast",
    "template",
    "this",
    "throw",
    "try",
    "typeid",
    "typename",
    "using",
    "virtual",
    "wchar_t",
    "xor",
    "xor_eq",
];

/// A C identifier that is guaranteed not to collide with C/C++ keywords.
///
/// On construction, if the name matches a reserved keyword, a trailing `_`
/// is appended (e.g. `"register"` → `"register_"`).
#[derive(Clone)]
pub struct CIdentifier(String);

impl CIdentifier {
    /// Create a new C identifier, escaping C/C++ keywords by appending `_`.
    pub fn new(name: impl Into<String>) -> Self {
        let name = name.into();
        if C_KEYWORDS.contains(&name.as_str()) {
            Self(format!("{name}_"))
        } else {
            Self(name)
        }
    }

    /// Return the (possibly escaped) identifier as a string slice.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

impl fmt::Debug for CIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_tuple("CIdentifier").field(&self.0).finish()
    }
}

impl fmt::Display for CIdentifier {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.0)
    }
}

/// A user-defined type that needs a C declaration in the header.
#[derive(Clone, Debug)]
pub struct CTypeDefinition {
    /// The C name for this type (last path segment from PathType::base_type),
    /// or a custom name provided via `#[cheadergen::config(rename = "...")]`.
    pub name: String,
    /// The original C name before rename, if the type was renamed.
    /// Used to build a rename map so function signatures reference the correct name.
    pub original_name: Option<String>,
    /// Whether this is an opaque forward declaration or a full struct definition.
    pub kind: CTypeKind,
    /// The global rustdoc item ID, used for doc comment lookup at codegen time.
    /// Pairs the crate-local rustdoc ID with the package that defines the type.
    pub rustdoc_id: Option<GlobalItemId>,
    /// The package that defines this type (from `PathType::package_id`).
    /// Used for partitioning types across per-crate header files.
    pub defining_package: PackageId,
    /// `true` when this is a monomorphized generic instantiation
    /// (the originating `PathType` had concrete generic arguments).
    /// Generic instantiations are placed in the consuming crate's header
    /// and wrapped in `#ifndef` guards to handle potential duplication.
    pub is_generic_instantiation: bool,
}

/// The kind of C type definition to emit.
#[derive(Clone, Debug)]
pub enum CTypeKind {
    /// Emit only a forward declaration (`struct Foo;` / `typedef struct Foo Foo;`).
    OpaqueStruct,
    /// Emit only a forward declaration (`union Foo;` / `typedef union Foo Foo;`).
    OpaqueUnion,
    /// Emit a full struct definition with fields.
    Struct(CStructDef),
    /// Emit a plain C union definition with fields.
    Union(CUnionDef),
    /// Emit a C-like enum (no data variants).
    FieldlessEnum(CFieldlessEnumDef),
    /// Emit a tagged union (enum with data variants).
    TaggedUnion(CTaggedUnionDef),
    /// Emit a typedef to the wrapped type (`typedef <inner> <name>;`).
    ///
    /// Used for both `#[repr(transparent)]` structs and Rust `type` aliases.
    Typedef(CTypedefDef),
}

/// A type emitted as a C typedef (`typedef <inner> <name>;`).
///
/// Produced from `#[repr(transparent)]` structs (single non-ZST field)
/// and Rust `type` aliases.
#[derive(Clone, Debug)]
pub struct CTypedefDef {
    /// The resolved inner type that the typedef aliases.
    pub inner: Type,
}

/// A resolved `#[repr(C)]` struct with its fields.
#[derive(Clone, Debug)]
pub struct CStructDef {
    /// The fields of the struct, in declaration order.
    pub fields: Vec<CStructField>,
}

/// A resolved `#[repr(C)]` union with its fields.
#[derive(Clone, Debug)]
pub struct CUnionDef {
    /// The fields of the union, in declaration order.
    pub fields: Vec<CStructField>,
}

/// A single field of a C struct.
#[derive(Clone, Debug)]
pub struct CStructField {
    /// The C field name (Rust name for plain structs, `m0`/`m1`/... for tuple structs).
    pub name: CIdentifier,
    /// The resolved type of this field.
    pub type_: Type,
    /// If set, emit this field as a C bitfield with the given width in bits.
    pub bitfield_width: Option<u64>,
}

/// A C-like enum (all variants are fieldless).
#[derive(Clone, Debug)]
pub struct CFieldlessEnumDef {
    pub repr: CEnumRepr,
    pub variants: Vec<CEnumVariant>,
    /// Prefix each variant name with the enum name (e.g. `Status_Ok`).
    pub prefix_with_name: bool,
}

/// A single variant of a fieldless enum.
#[derive(Clone, Debug)]
pub struct CEnumVariant {
    pub name: CIdentifier,
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
#[derive(Clone, Debug)]
pub enum CEnumRepr {
    /// `#[repr(C)]` only — use a plain C enum.
    C,
    /// `#[repr(uN)]` or `#[repr(C, uN)]` — emit enum constants + typedef to int type.
    Int {
        is_repr_c: bool,
        int_type: ReprIntType,
    },
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
#[derive(Clone, Debug)]
pub struct CTaggedUnionDef {
    pub repr: CEnumRepr,
    /// When true, variant names in the tag enum are prefixed with the enum name.
    pub prefix_with_name: bool,
    pub variants: Vec<CTaggedVariant>,
}

/// A single variant of a tagged union.
#[derive(Clone, Debug)]
pub struct CTaggedVariant {
    pub name: String,
    pub body: Option<CTaggedVariantBody>,
    pub discriminant: Option<String>,
}

/// The body (fields) of a tagged union variant.
#[derive(Clone, Debug)]
pub struct CTaggedVariantBody {
    pub fields: Vec<CStructField>,
}

/// Whether a type appears by value or only behind a pointer/reference.
#[derive(Clone, Copy, PartialEq, Eq)]
pub(super) enum TypeUsage {
    /// Type appears by value in a signature or struct field.
    ByValue,
    /// Type only appears behind a pointer or reference.
    BehindPointer,
}

/// Walk all types in function signatures and static items, collecting unique
/// path types that need C declarations in the generated header.
///
/// Types used directly (not only behind pointers) and marked `#[repr(C)]` get
/// full struct definitions. All others get forward declarations.
///
/// Exported types from [`AnnotatedExports`] are seeded before the fixed-point
/// resolution loop: `full` exports as by-value, `opaque` exports as
/// behind-pointer (forward declaration only).
pub fn collect_type_definitions(
    extern_items: &ExternItems,
    collection: &Collection,
    enum_prefix_with_name: bool,
    overrides: &PackageTypeOverrides,
    diagnostics: &mut DiagnosticSink,
) -> anyhow::Result<Vec<CTypeDefinition>> {
    let exports = exported_via_annotations(&extern_items.package_id, collection, diagnostics)
        .map_err(|e| anyhow::anyhow!(e))?;

    let mut seen: HashMap<CCanonicalType, TypeUsage> = HashMap::new();

    // Seed the queue with types that were annotated using `#[cheadergen::export]`.
    for ct in &exports.full {
        seen.entry(ct.clone())
            .and_modify(|u| *u = TypeUsage::ByValue)
            .or_insert(TypeUsage::ByValue);
    }
    for ct in &exports.opaque {
        seen.insert(ct.clone(), TypeUsage::BehindPointer);
    }

    // Do a first pass over extern items to collect paths from their input/output types.
    for func in &extern_items.fns {
        for input in &func.header.inputs {
            collect_paths_from_type(
                &input.type_,
                TypeUsage::ByValue,
                &mut seen,
                collection,
                overrides,
            );
        }
        if let Some(output) = &func.header.output {
            collect_paths_from_type(output, TypeUsage::ByValue, &mut seen, collection, overrides);
        }
    }
    for s in &extern_items.statics {
        collect_paths_from_type(
            &s.type_,
            TypeUsage::ByValue,
            &mut seen,
            collection,
            overrides,
        );
    }

    // Resolve struct/enum/union fields for directly-used types. This may discover new
    // transitive types that also need definitions.
    resolve_all_type_definitions(
        &mut seen,
        collection,
        enum_prefix_with_name,
        overrides,
        diagnostics,
    )
}

/// Walk all types across **multiple** targets' extern items, collecting the union of
/// path types that need C declarations across all generated headers.
///
/// This is the partitioned-mode counterpart of [`collect_type_definitions`]: it seeds
/// the `seen` map from every target's functions, statics, and annotated exports, then
/// runs the same fixed-point resolution.
pub fn collect_type_definitions_multi(
    target_extern_items: &[(PackageId, ExternItems)],
    collection: &Collection,
    enum_prefix_with_name: bool,
    overrides: &PackageTypeOverrides,
    diagnostics: &mut DiagnosticSink,
) -> anyhow::Result<Vec<CTypeDefinition>> {
    let mut seen: HashMap<CCanonicalType, TypeUsage> = HashMap::new();

    for (package_id, extern_items) in target_extern_items {
        // Seed annotated exports for each target.
        let exports = exported_via_annotations(package_id, collection, diagnostics)
            .map_err(|e| anyhow::anyhow!(e))?;
        for ct in &exports.full {
            seen.entry(ct.clone())
                .and_modify(|u| *u = TypeUsage::ByValue)
                .or_insert(TypeUsage::ByValue);
        }
        for ct in &exports.opaque {
            seen.entry(ct.clone()).or_insert(TypeUsage::BehindPointer);
        }

        // Seed from extern items.
        for func in &extern_items.fns {
            for input in &func.header.inputs {
                collect_paths_from_type(
                    &input.type_,
                    TypeUsage::ByValue,
                    &mut seen,
                    collection,
                    overrides,
                );
            }
            if let Some(output) = &func.header.output {
                collect_paths_from_type(
                    output,
                    TypeUsage::ByValue,
                    &mut seen,
                    collection,
                    overrides,
                );
            }
        }
        for s in &extern_items.statics {
            collect_paths_from_type(
                &s.type_,
                TypeUsage::ByValue,
                &mut seen,
                collection,
                overrides,
            );
        }
    }

    resolve_all_type_definitions(
        &mut seen,
        collection,
        enum_prefix_with_name,
        overrides,
        diagnostics,
    )
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
        Type::Path(p) | Type::TypeAlias(p) => {
            let base = p.base_type.last().expect("empty path");
            let type_args: Vec<String> = p
                .generic_arguments
                .iter()
                .filter_map(|arg| match arg {
                    GenericArgument::TypeParameter(t) => Some(c_type_name(t)),
                    GenericArgument::Lifetime(_) => None,
                    GenericArgument::Const(c) => Some(c.value.clone()),
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
            // Function pointers are emitted inline (not by name); this path is
            // reached when computing names for wrapper types like Option<fn()>.
            "fn".to_owned()
        }
    }
}

/// Returns `true` if the path type has at least one concrete generic argument
/// (a type parameter or const value), indicating this is a monomorphized instantiation
/// rather than a plain non-generic type.
fn has_concrete_generic_args(path_type: &PathType) -> bool {
    path_type.generic_arguments.iter().any(|a| {
        matches!(
            a,
            GenericArgument::TypeParameter(_) | GenericArgument::Const(_)
        )
    })
}

/// Extract the inner `PathType` from a `CCanonicalType`.
///
/// Panics if the canonical type is not a `Type::Path` or `Type::TypeAlias`.
fn canonical_type_to_path(ct: &CCanonicalType) -> PathType {
    match ct.inner() {
        Type::Path(p) | Type::TypeAlias(p) => p.clone(),
        _ => unreachable!("AnnotatedExports should only contain path types"),
    }
}

/// Traverse a type to collect other types that are transitively used by it.
pub(super) fn collect_paths_from_type(
    ty: &Type,
    usage: TypeUsage,
    seen: &mut HashMap<CCanonicalType, TypeUsage>,
    collection: &Collection,
    overrides: &PackageTypeOverrides,
) {
    match ty {
        Type::Path(p) | Type::TypeAlias(p) => {
            // FFI primitive types (c_int, c_void, etc.) map directly to native
            // C types — never collect them so no typedef or opaque is emitted.
            if ffi_primitive_to_c(p).is_some() {
                return;
            }

            // Package-level skip: don't collect the type at all,
            // same as annotation-level `#[cheadergen::skip]`.
            if overrides.skipped.contains(&p.package_id) {
                return;
            }

            let annotation = collection
                .get_annotated_items(&p.package_id)
                .and_then(|ann| ann.get(&p.rustdoc_id?));

            let is_skip = annotation.as_ref().is_some_and(|a| a.skip);
            if is_skip {
                return;
            }

            // Package-level opaque: never upgrade to ByValue,
            // same as annotation-level `#[cheadergen::export(opaque)]`.
            let must_stay_opaque = overrides.opaque.contains(&p.package_id)
                || annotation.as_ref().and_then(|ann| ann.export) == Some(ExportMode::Opaque);

            let canonical = CCanonicalType::new(ty.canonicalize(collection));
            let entry = seen.entry(canonical).or_insert(TypeUsage::BehindPointer);
            if usage == TypeUsage::ByValue && !must_stay_opaque {
                *entry = TypeUsage::ByValue;
            }
        }
        Type::Reference(r) => collect_paths_from_type(
            &r.inner,
            TypeUsage::BehindPointer,
            seen,
            collection,
            overrides,
        ),
        Type::RawPointer(r) => collect_paths_from_type(
            &r.inner,
            TypeUsage::BehindPointer,
            seen,
            collection,
            overrides,
        ),
        Type::Tuple(t) => {
            for element in &t.elements {
                collect_paths_from_type(element, usage, seen, collection, overrides);
            }
        }
        Type::Slice(s) => {
            collect_paths_from_type(&s.element_type, usage, seen, collection, overrides)
        }
        Type::Array(a) => {
            collect_paths_from_type(&a.element_type, usage, seen, collection, overrides)
        }
        Type::FunctionPointer(fp) => {
            for input in &fp.inputs {
                collect_paths_from_type(
                    &input.type_,
                    TypeUsage::BehindPointer,
                    seen,
                    collection,
                    overrides,
                );
            }
            if let Some(output) = &fp.output {
                collect_paths_from_type(
                    output,
                    TypeUsage::BehindPointer,
                    seen,
                    collection,
                    overrides,
                );
            }
        }
        Type::ScalarPrimitive(_) | Type::Generic(_) => {}
    }
}

/// Zero-sized types that should be skipped when emitting struct fields.
pub(super) fn is_zst_type(ty: &Type) -> bool {
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
                base_type
                    .iter()
                    .map(String::as_str)
                    .collect::<Vec<_>>()
                    .as_slice(),
                ["core" | "std", "marker", "PhantomData" | "PhantomPinned"]
            )
        }
        _ => false,
    }
}

/// If `path` is a `std::ffi` / `core::ffi` primitive type alias, return
/// the native C type name it maps to. Returns `None` for non-FFI types.
pub fn ffi_primitive_to_c(path: &PathType) -> Option<&'static str> {
    let pkg = path.package_id.repr();
    if pkg != CORE_PACKAGE_ID_REPR && pkg != STD_PACKAGE_ID_REPR {
        return None;
    }
    match path.base_type.last().map(String::as_str)? {
        "c_char" => Some("char"),
        "c_schar" => Some("signed char"),
        "c_uchar" => Some("unsigned char"),
        "c_short" => Some("short"),
        "c_ushort" => Some("unsigned short"),
        "c_int" => Some("int"),
        "c_uint" => Some("unsigned int"),
        "c_long" => Some("long"),
        "c_ulong" => Some("unsigned long"),
        "c_longlong" => Some("long long"),
        "c_ulonglong" => Some("unsigned long long"),
        "c_float" => Some("float"),
        "c_double" => Some("double"),
        "c_void" => Some("void"),
        _ => None,
    }
}

/// Look up any `rename` annotation for a type by its rustdoc ID.
fn annotation_rename(collection: &Collection, path_type: &PathType) -> Option<String> {
    let ann = collection.get_annotated_items(&path_type.package_id)?;
    let id = path_type.rustdoc_id?;
    ann.get(&id)?.rename.clone()
}

/// Look up `prefix_with_name` annotation for a type, falling back to
/// the global config value.
fn annotation_prefix_with_name(
    collection: &Collection,
    path_type: &PathType,
    global_default: bool,
) -> bool {
    collection
        .get_annotated_items(&path_type.package_id)
        .and_then(|ann| {
            let id = path_type.rustdoc_id?;
            ann.get(&id)?.prefix_with_name
        })
        .unwrap_or(global_default)
}

/// Resolve all collected types into `CTypeDefinition`s, iterating to a fixed
/// point to discover transitive field types.
fn resolve_all_type_definitions(
    seen: &mut HashMap<CCanonicalType, TypeUsage>,
    collection: &Collection,
    enum_prefix_with_name: bool,
    overrides: &PackageTypeOverrides,
    diagnostics: &mut DiagnosticSink,
) -> anyhow::Result<Vec<CTypeDefinition>> {
    // Phase 1: fixed-point loop over directly-used types.
    // By resolving all direct uses first, we ensure that a type initially seen
    // behind a pointer gets upgraded to direct when a struct field references it
    // by value — before we emit any opaques.
    let mut resolved: HashMap<CCanonicalType, CTypeDefinition> = HashMap::new();
    loop {
        let direct: Vec<(CCanonicalType, PathType)> = seen
            .iter()
            .filter(|(ct, usage)| **usage == TypeUsage::ByValue && !resolved.contains_key(*ct))
            .map(|(ct, _)| (ct.clone(), canonical_type_to_path(ct)))
            .collect();

        if direct.is_empty() {
            break;
        }

        for (canonical, path_type) in direct {
            let per_type_prefix =
                annotation_prefix_with_name(collection, &path_type, enum_prefix_with_name);

            let default_name = c_type_name(&Type::Path(path_type.clone()));
            let renamed = annotation_rename(collection, &path_type);
            let original_name = renamed.as_ref().map(|_| default_name.clone());
            let name = renamed.unwrap_or(default_name);
            let kind =
                resolve_type_kind(&name, &path_type, collection, per_type_prefix, diagnostics)?;

            // Discover transitive field types from full definitions.
            match &kind {
                CTypeKind::Struct(def) => {
                    for field in &def.fields {
                        collect_paths_from_type(
                            &field.type_,
                            TypeUsage::ByValue,
                            seen,
                            collection,
                            overrides,
                        );
                    }
                }
                CTypeKind::Union(def) => {
                    for field in &def.fields {
                        collect_paths_from_type(
                            &field.type_,
                            TypeUsage::ByValue,
                            seen,
                            collection,
                            overrides,
                        );
                    }
                }
                CTypeKind::TaggedUnion(def) => {
                    for variant in &def.variants {
                        if let Some(ref body) = variant.body {
                            for field in &body.fields {
                                collect_paths_from_type(
                                    &field.type_,
                                    TypeUsage::ByValue,
                                    seen,
                                    collection,
                                    overrides,
                                );
                            }
                        }
                    }
                }
                CTypeKind::Typedef(def) => {
                    collect_paths_from_type(
                        &def.inner,
                        TypeUsage::ByValue,
                        seen,
                        collection,
                        overrides,
                    );
                }
                CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion | CTypeKind::FieldlessEnum(_) => {}
            }

            let is_generic_instantiation = has_concrete_generic_args(&path_type);
            resolved.insert(
                canonical,
                CTypeDefinition {
                    name,
                    original_name,
                    kind,
                    rustdoc_id: path_type
                        .rustdoc_id
                        .map(|id| GlobalItemId::new(id, path_type.package_id.clone())),
                    defining_package: path_type.package_id.clone(),
                    is_generic_instantiation,
                },
            );
        }
    }

    // Phase 2: pointer-only types — fixed-point loop that resolves each type's
    // real kind, preserves typedefs (aliases and `repr(transparent)` wrappers)
    // verbatim, and downgrades genuine compound types to opaque forward
    // declarations. A typedef enqueues its inner type for pointer-only
    // resolution in a subsequent iteration so transitive references stay in the
    // collection.
    loop {
        let unresolved: Vec<(CCanonicalType, PathType)> = seen
            .iter()
            .filter(|(ct, _)| !resolved.contains_key(*ct))
            .map(|(ct, _)| (ct.clone(), canonical_type_to_path(ct)))
            .collect();

        if unresolved.is_empty() {
            break;
        }

        for (canonical, path_type) in unresolved {
            let per_type_prefix =
                annotation_prefix_with_name(collection, &path_type, enum_prefix_with_name);

            let default_name = c_type_name(&Type::Path(path_type.clone()));
            let renamed = annotation_rename(collection, &path_type);
            let original_name = renamed.as_ref().map(|_| default_name.clone());
            let name = renamed.unwrap_or(default_name);
            let rustdoc_id = path_type
                .rustdoc_id
                .map(|id| GlobalItemId::new(id, path_type.package_id.clone()));

            let resolved_kind = resolve_type_kind(
                &name,
                &path_type,
                collection,
                per_type_prefix,
                diagnostics,
            )?;

            let kind = match resolved_kind {
                CTypeKind::Typedef(def) => {
                    collect_paths_from_type(
                        &def.inner,
                        TypeUsage::BehindPointer,
                        seen,
                        collection,
                        overrides,
                    );
                    CTypeKind::Typedef(def)
                }
                CTypeKind::Union(_) | CTypeKind::OpaqueUnion => CTypeKind::OpaqueUnion,
                CTypeKind::Struct(_)
                | CTypeKind::TaggedUnion(_)
                | CTypeKind::FieldlessEnum(_)
                | CTypeKind::OpaqueStruct => CTypeKind::OpaqueStruct,
            };

            let is_generic_instantiation = has_concrete_generic_args(&path_type);
            resolved.insert(
                canonical,
                CTypeDefinition {
                    name,
                    original_name,
                    kind,
                    rustdoc_id,
                    defining_package: path_type.package_id.clone(),
                    is_generic_instantiation,
                },
            );
        }
    }

    let defs: Vec<CTypeDefinition> = resolved.into_values().collect();
    Ok(defs)
}
