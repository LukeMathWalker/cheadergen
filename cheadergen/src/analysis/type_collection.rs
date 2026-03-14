use std::collections::HashMap;
use std::fmt;

use rustdoc_ir::{FreeFunction, GenericArgument, PathType, ScalarPrimitive, Type};
use rustdoc_processor::indexing::CrateIndexer;
use rustdoc_processor::{CORE_PACKAGE_ID_REPR, CrateCollection, GlobalItemId, STD_PACKAGE_ID_REPR};
use rustdoc_types::ItemEnum;

use super::type_resolution::resolve_type_kind;
use crate::static_item::StaticItem;

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
pub struct CTypeDefinition {
    /// The C name for this type (last path segment from PathType::base_type),
    /// or a custom name provided via `#[export(name = "...")]`.
    pub name: String,
    /// Whether this is an opaque forward declaration or a full struct definition.
    pub kind: CTypeKind,
    /// The global rustdoc item ID, used for doc comment lookup at codegen time.
    /// Pairs the crate-local rustdoc ID with the package that defines the type.
    pub rustdoc_id: Option<GlobalItemId>,
}

/// The kind of C type definition to emit.
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
pub struct CTypedefDef {
    /// The resolved inner type that the typedef aliases.
    pub inner: Type,
}

/// A resolved `#[repr(C)]` struct with its fields.
pub struct CStructDef {
    /// The fields of the struct, in declaration order.
    pub fields: Vec<CStructField>,
}

/// A resolved `#[repr(C)]` union with its fields.
pub struct CUnionDef {
    /// The fields of the union, in declaration order.
    pub fields: Vec<CStructField>,
}

/// A single field of a C struct.
pub struct CStructField {
    /// The C field name (Rust name for plain structs, `m0`/`m1`/... for tuple structs).
    pub name: CIdentifier,
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
pub fn collect_type_definitions<I: CrateIndexer>(
    functions: &[FreeFunction],
    statics: &[StaticItem],
    collection: &CrateCollection<I>,
    enum_prefix_with_name: bool,
) -> anyhow::Result<Vec<CTypeDefinition>> {
    let mut seen: HashMap<PathType, TypeUsage> = HashMap::new();
    for func in functions {
        for input in &func.header.inputs {
            collect_paths_from_type(&input.type_, TypeUsage::ByValue, &mut seen);
        }
        if let Some(output) = &func.header.output {
            collect_paths_from_type(output, TypeUsage::ByValue, &mut seen);
        }
    }
    for s in statics {
        collect_paths_from_type(&s.type_, TypeUsage::ByValue, &mut seen);
    }

    // Resolve struct fields for directly-used types. This may discover new
    // transitive types that also need definitions.
    resolve_all_type_definitions(&mut seen, collection, enum_prefix_with_name)
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

pub(super) fn collect_paths_from_type(
    ty: &Type,
    usage: TypeUsage,
    seen: &mut HashMap<PathType, TypeUsage>,
) {
    match ty {
        Type::Path(p) | Type::TypeAlias(p) => {
            let entry = seen.entry(p.clone()).or_insert(TypeUsage::BehindPointer);
            if usage == TypeUsage::ByValue {
                *entry = TypeUsage::ByValue;
            }
        }
        Type::Reference(r) => collect_paths_from_type(&r.inner, TypeUsage::BehindPointer, seen),
        Type::RawPointer(r) => collect_paths_from_type(&r.inner, TypeUsage::BehindPointer, seen),
        Type::Tuple(t) => {
            for element in &t.elements {
                collect_paths_from_type(element, usage, seen);
            }
        }
        Type::Slice(s) => collect_paths_from_type(&s.element_type, usage, seen),
        Type::Array(a) => collect_paths_from_type(&a.element_type, usage, seen),
        Type::FunctionPointer(fp) => {
            for input in &fp.inputs {
                collect_paths_from_type(input, TypeUsage::BehindPointer, seen);
            }
            if let Some(output) = &fp.output {
                collect_paths_from_type(output, TypeUsage::BehindPointer, seen);
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

/// Resolve all collected types into `CTypeDefinition`s, iterating to a fixed
/// point to discover transitive field types.
fn resolve_all_type_definitions<I: CrateIndexer>(
    seen: &mut HashMap<PathType, TypeUsage>,
    collection: &CrateCollection<I>,
    enum_prefix_with_name: bool,
) -> anyhow::Result<Vec<CTypeDefinition>> {
    // Phase 1: fixed-point loop over directly-used types.
    // By resolving all direct uses first, we ensure that a type initially seen
    // behind a pointer gets upgraded to direct when a struct field references it
    // by value — before we emit any opaques.
    let mut resolved: HashMap<PathType, CTypeDefinition> = HashMap::new();
    loop {
        let direct: Vec<PathType> = seen
            .iter()
            .filter(|(pt, usage)| **usage == TypeUsage::ByValue && !resolved.contains_key(*pt))
            .map(|(pt, _)| pt.clone())
            .collect();

        if direct.is_empty() {
            break;
        }

        for path_type in direct {
            let name = c_type_name(&Type::Path(path_type.clone()));
            let kind = resolve_type_kind(&name, &path_type, collection, enum_prefix_with_name)?;

            // Discover transitive field types from full definitions.
            match &kind {
                CTypeKind::Struct(def) => {
                    for field in &def.fields {
                        collect_paths_from_type(&field.type_, TypeUsage::ByValue, seen);
                    }
                }
                CTypeKind::Union(def) => {
                    for field in &def.fields {
                        collect_paths_from_type(&field.type_, TypeUsage::ByValue, seen);
                    }
                }
                CTypeKind::TaggedUnion(def) => {
                    for variant in &def.variants {
                        if let Some(ref body) = variant.body {
                            for field in &body.fields {
                                collect_paths_from_type(&field.type_, TypeUsage::ByValue, seen);
                            }
                        }
                    }
                }
                CTypeKind::Typedef(def) => {
                    collect_paths_from_type(&def.inner, TypeUsage::ByValue, seen);
                }
                CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion | CTypeKind::FieldlessEnum(_) => {}
            }

            resolved.insert(
                path_type.clone(),
                CTypeDefinition {
                    name,
                    kind,
                    rustdoc_id: path_type
                        .rustdoc_id
                        .map(|id| GlobalItemId::new(id, path_type.package_id.clone())),
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
        let rustdoc_id = path_type
            .rustdoc_id
            .map(|id| GlobalItemId::new(id, path_type.package_id.clone()));
        // Determine if the underlying item is a union so we emit the correct tag.
        let is_union = rustdoc_id
            .as_ref()
            .map(|gid| collection.get_item_by_global_type_id(gid))
            .is_some_and(|item| matches!(item.inner, ItemEnum::Union(_)));
        let kind = if is_union {
            CTypeKind::OpaqueUnion
        } else {
            CTypeKind::OpaqueStruct
        };
        resolved.insert(
            path_type,
            CTypeDefinition {
                name,
                kind,
                rustdoc_id,
            },
        );
    }

    let defs: Vec<CTypeDefinition> = resolved.into_values().collect();
    Ok(defs)
}
