use std::sync::Arc;

use guppy::PackageId;
use rustdoc_ir::{GenericArgument, PathType, Type};
use rustdoc_processor::indexing::CrateIndexer;
use rustdoc_processor::{CrateCollection, GlobalItemId};
use rustdoc_resolver::{GenericBindings, TypeAliasResolution, resolve_type};
use rustdoc_types::{Attribute, AttributeRepr, ItemEnum, ReprKind, StructKind, VariantKind};

use super::type_collection::{
    CEnumRepr, CEnumVariant, CFieldlessEnumDef, CIdentifier, CStructDef, CStructField,
    CTaggedUnionDef, CTaggedVariant, CTaggedVariantBody, CTypeKind, CTypedefDef, CUnionDef,
    ReprIntType, is_zst_type,
};

/// Attempt to resolve a directly-used type into a full definition.
///
/// Returns an opaque variant (with a warning on stderr) if the type cannot
/// be fully defined — e.g. missing `rustdoc_id`, not `#[repr(C)]`, etc.
pub(super) fn resolve_type_kind<I: CrateIndexer>(
    name: &str,
    path_type: &PathType,
    collection: &CrateCollection<I>,
    enum_prefix_with_name: bool,
) -> anyhow::Result<CTypeKind> {
    let Some(id) = &path_type.rustdoc_id else {
        eprintln!("warning: type `{name}` has no rustdoc ID; emitting forward declaration");
        return Ok(CTypeKind::OpaqueStruct);
    };

    let global_id = GlobalItemId::new(*id, path_type.package_id.clone());
    let item = collection.get_item_by_global_type_id(&global_id);

    match &item.inner {
        ItemEnum::Struct(struct_def) => {
            resolve_struct_kind(name, struct_def, &item.attrs, path_type, collection)
        }
        ItemEnum::Union(union_def) => {
            resolve_union_kind(name, union_def, &item.attrs, path_type, collection)
        }
        ItemEnum::Enum(enum_def) => {
            resolve_enum_kind(name, enum_def, &item.attrs, path_type, collection, enum_prefix_with_name)
        }
        ItemEnum::TypeAlias(type_alias) => {
            resolve_type_alias_kind(name, type_alias, path_type, collection)
        }
        _ => {
            eprintln!(
                "warning: type `{name}` is not a struct, union, enum, or type alias; emitting forward declaration"
            );
            Ok(CTypeKind::OpaqueStruct)
        }
    }
}

/// Build generic bindings for a type with type parameters, returning
/// `Err(())` (with a warning) if the type cannot be monomorphized.
/// Callers map the error to the appropriate opaque variant.
fn setup_generic_bindings(
    name: &str,
    generics: &rustdoc_types::Generics,
    path_type: &PathType,
) -> Result<GenericBindings, ()> {
    let mut bindings = if has_type_params(generics) {
        match build_generic_bindings(generics, &path_type.generic_arguments) {
            Some(bindings) => bindings,
            None => {
                eprintln!(
                    "warning: type `{name}` has generic type parameters; emitting forward declaration"
                );
                return Err(());
            }
        }
    } else {
        GenericBindings::default()
    };

    // Bind `Self` to the containing type so that `*const Self` / `*mut Self`
    // fields resolve to the correct concrete type.
    bindings
        .types
        .insert("Self".into(), Type::Path(path_type.clone()));

    Ok(bindings)
}

/// Resolve a struct into a `CTypeKind`.
fn resolve_struct_kind<I: CrateIndexer>(
    name: &str,
    struct_def: &rustdoc_types::Struct,
    attrs: &[Attribute],
    path_type: &PathType,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
    // Check for #[repr(C)] or #[repr(transparent)].
    let is_repr_c = attrs.iter().any(|attr| {
        matches!(
            attr,
            Attribute::Repr(AttributeRepr {
                kind: ReprKind::C,
                ..
            })
        )
    });
    let is_repr_transparent = attrs.iter().any(|attr| {
        matches!(
            attr,
            Attribute::Repr(AttributeRepr {
                kind: ReprKind::Transparent,
                ..
            })
        )
    });

    if !is_repr_c && !is_repr_transparent {
        eprintln!("warning: type `{name}` is not #[repr(C)]; emitting opaque forward declaration");
        return Ok(CTypeKind::OpaqueStruct);
    }

    let generic_bindings = match setup_generic_bindings(name, &struct_def.generics, path_type) {
        Ok(bindings) => bindings,
        Err(()) => return Ok(CTypeKind::OpaqueStruct),
    };

    if is_repr_transparent {
        return resolve_transparent_struct(
            name,
            struct_def,
            &generic_bindings,
            path_type,
            collection,
        );
    }

    match &struct_def.kind {
        StructKind::Plain { fields, .. } => {
            let c_fields =
                resolve_plain_fields(fields, &generic_bindings, &path_type.package_id, collection)?;
            Ok(CTypeKind::Struct(CStructDef { fields: c_fields }))
        }
        StructKind::Tuple(fields) => {
            let c_fields = resolve_tuple_fields(
                fields,
                &generic_bindings,
                &path_type.package_id,
                collection,
            )?;
            Ok(CTypeKind::Struct(CStructDef { fields: c_fields }))
        }
        StructKind::Unit => Ok(CTypeKind::Struct(CStructDef { fields: Vec::new() })),
    }
}

/// Resolve a `#[repr(transparent)]` struct into a `CTypeKind`.
///
/// If exactly one non-ZST field exists, emits a typedef. If zero non-ZST
/// fields exist, falls back to an empty struct.
fn resolve_transparent_struct<I: CrateIndexer>(
    name: &str,
    struct_def: &rustdoc_types::Struct,
    generic_bindings: &rustdoc_resolver::GenericBindings,
    path_type: &PathType,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
    let c_fields = match &struct_def.kind {
        StructKind::Plain { fields, .. } => {
            resolve_plain_fields(fields, generic_bindings, &path_type.package_id, collection)?
        }
        StructKind::Tuple(fields) => {
            resolve_tuple_fields(fields, generic_bindings, &path_type.package_id, collection)?
        }
        StructKind::Unit => Vec::new(),
    };

    match c_fields.len() {
        0 => Ok(CTypeKind::Struct(CStructDef { fields: Vec::new() })),
        1 => Ok(CTypeKind::Typedef(CTypedefDef {
            inner: c_fields.into_iter().next().unwrap().type_,
        })),
        n => {
            eprintln!(
                "warning: repr(transparent) type `{name}` has {n} non-ZST fields; \
                 emitting opaque forward declaration"
            );
            Ok(CTypeKind::OpaqueStruct)
        }
    }
}

/// Resolve a Rust `type` alias into a `CTypeKind::Typedef`.
fn resolve_type_alias_kind<I: CrateIndexer>(
    name: &str,
    type_alias: &rustdoc_types::TypeAlias,
    path_type: &PathType,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
    let generic_bindings = match setup_generic_bindings(name, &type_alias.generics, path_type) {
        Ok(bindings) => bindings,
        Err(()) => return Ok(CTypeKind::OpaqueStruct),
    };

    // Resolve the aliased type fully (resolve through nested aliases).
    let resolved = resolve_type(
        &type_alias.type_,
        &path_type.package_id,
        collection,
        &generic_bindings,
        TypeAliasResolution::ResolveThrough,
    )
    .map_err(|e| {
        anyhow::anyhow!("Failed to resolve type alias `{name}`: {}", Arc::new(e))
    })?;

    Ok(CTypeKind::Typedef(CTypedefDef { inner: resolved }))
}

/// Resolve a union into a `CTypeKind`.
fn resolve_union_kind<I: CrateIndexer>(
    name: &str,
    union_def: &rustdoc_types::Union,
    attrs: &[Attribute],
    path_type: &PathType,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
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
        eprintln!("warning: union `{name}` is not #[repr(C)]; emitting opaque forward declaration");
        return Ok(CTypeKind::OpaqueUnion);
    }

    let generic_bindings = match setup_generic_bindings(name, &union_def.generics, path_type) {
        Ok(bindings) => bindings,
        Err(()) => return Ok(CTypeKind::OpaqueUnion),
    };

    let c_fields = resolve_plain_fields(&union_def.fields, &generic_bindings, &path_type.package_id, collection)?;
    Ok(CTypeKind::Union(CUnionDef { fields: c_fields }))
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
    collection: &CrateCollection<I>,
    enum_prefix_with_name: bool,
) -> anyhow::Result<CTypeKind> {
    let Some(repr) = extract_enum_repr(attrs)? else {
        eprintln!(
            "warning: enum `{name}` has no C-compatible repr; emitting opaque forward declaration"
        );
        return Ok(CTypeKind::OpaqueStruct);
    };

    let generic_bindings = match setup_generic_bindings(name, &enum_def.generics, path_type) {
        Ok(bindings) => bindings,
        Err(()) => return Ok(CTypeKind::OpaqueStruct),
    };

    let package_id = &path_type.package_id;

    // Classify: all variants plain → fieldless, otherwise → tagged union.
    let mut all_plain = true;
    for variant_id in &enum_def.variants {
        let global_id = GlobalItemId::new(*variant_id, package_id.clone());
        let variant_item = collection.get_item_by_global_type_id(&global_id);
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
        resolve_fieldless_enum(name, enum_def, repr, package_id, collection)
    } else {
        resolve_tagged_union(
            name,
            enum_def,
            repr,
            &generic_bindings,
            package_id,
            collection,
            enum_prefix_with_name,
        )
    }
}

/// Resolve a fieldless enum.
fn resolve_fieldless_enum<I: CrateIndexer>(
    name: &str,
    enum_def: &rustdoc_types::Enum,
    repr: CEnumRepr,
    package_id: &PackageId,
    collection: &CrateCollection<I>,
) -> anyhow::Result<CTypeKind> {
    let mut variants = Vec::new();
    for variant_id in &enum_def.variants {
        let global_id = GlobalItemId::new(*variant_id, package_id.clone());
        let variant_item = collection.get_item_by_global_type_id(&global_id);
        let ItemEnum::Variant(variant) = &variant_item.inner else {
            anyhow::bail!("Expected Variant for enum `{name}`");
        };
        let variant_name = variant_item.name.clone().unwrap_or_default();
        let discriminant = variant.discriminant.as_ref().map(|d| d.expr.clone());
        variants.push(CEnumVariant {
            name: CIdentifier::new(variant_name),
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
    package_id: &PackageId,
    collection: &CrateCollection<I>,
    enum_prefix_with_name: bool,
) -> anyhow::Result<CTypeKind> {
    let prefix_with_name = enum_prefix_with_name;

    let mut variants = Vec::new();
    for variant_id in &enum_def.variants {
        let global_id = GlobalItemId::new(*variant_id, package_id.clone());
        let variant_item = collection.get_item_by_global_type_id(&global_id);
        let ItemEnum::Variant(variant) = &variant_item.inner else {
            anyhow::bail!("Expected Variant for enum `{name}`");
        };
        let variant_name = variant_item.name.clone().unwrap_or_default();

        let body = match &variant.kind {
            VariantKind::Plain => None,
            VariantKind::Tuple(fields) => {
                let c_fields =
                    resolve_tuple_fields(fields, generic_bindings, package_id, collection)?;
                if c_fields.is_empty() {
                    None
                } else {
                    Some(CTaggedVariantBody { fields: c_fields })
                }
            }
            VariantKind::Struct { fields, .. } => {
                let c_fields =
                    resolve_plain_fields(fields, generic_bindings, package_id, collection)?;
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
    package_id: &PackageId,
    collection: &CrateCollection<I>,
) -> anyhow::Result<Vec<CStructField>> {
    let mut c_fields = Vec::new();
    for field_id in field_ids {
        let global_id = GlobalItemId::new(*field_id, package_id.clone());
        let field_item = collection.get_item_by_global_type_id(&global_id);
        let ItemEnum::StructField(ref raw_type) = field_item.inner else {
            anyhow::bail!("Expected StructField for id {:?}", field_id);
        };

        let field_name = field_item
            .name
            .clone()
            .unwrap_or_else(|| "<unnamed>".to_string());
        let resolved =
            resolve_type(raw_type, package_id, collection, generic_bindings, TypeAliasResolution::Preserve).map_err(|e| {
                anyhow::anyhow!("Failed to resolve field `{field_name}`: {}", Arc::new(e))
            })?;

        // Skip ZST fields (PhantomData, PhantomPinned, ()).
        if is_zst_type(&resolved) {
            continue;
        }

        c_fields.push(CStructField {
            name: CIdentifier::new(field_name),
            type_: resolved,
        });
    }
    Ok(c_fields)
}

/// Resolve tuple struct fields into C struct fields named `m0`, `m1`, etc.
fn resolve_tuple_fields<I: CrateIndexer>(
    fields: &[Option<rustdoc_types::Id>],
    generic_bindings: &GenericBindings,
    package_id: &PackageId,
    collection: &CrateCollection<I>,
) -> anyhow::Result<Vec<CStructField>> {
    let mut c_fields = Vec::new();
    let mut index = 0;
    for slot in fields {
        let Some(field_id) = slot else {
            // Private/hidden field — we can't emit the full struct.
            anyhow::bail!("Tuple struct has private fields");
        };
        let global_id = GlobalItemId::new(*field_id, package_id.clone());
        let field_item = collection.get_item_by_global_type_id(&global_id);
        let ItemEnum::StructField(ref raw_type) = field_item.inner else {
            anyhow::bail!("Expected StructField for tuple field id {:?}", field_id);
        };

        let resolved =
            resolve_type(raw_type, package_id, collection, generic_bindings, TypeAliasResolution::Preserve).map_err(|e| {
                anyhow::anyhow!("Failed to resolve tuple field m{index}: {}", Arc::new(e))
            })?;

        // Skip ZST fields.
        if is_zst_type(&resolved) {
            continue;
        }

        c_fields.push(CStructField {
            name: CIdentifier::new(format!("m{index}")),
            type_: resolved,
        });
        index += 1;
    }
    Ok(c_fields)
}
