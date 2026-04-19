use rustdoc_ir::{
    Array, CanonicalType, FunctionPointer, FunctionPointerInput, GenericArgument,
    GenericLifetimeParameter, Lifetime, PathType, RawPointer, Slice, Tuple, Type, TypeReference,
};

/// A canonical type identity for C header generation.
///
/// Wraps a [`rustdoc_ir::CanonicalType`] with all lifetime distinctions erased:
/// `'static`, `'a`, `'_`, and elided lifetimes all compare equal. C has no
/// notion of lifetimes, so any two Rust types that differ only in their
/// lifetime arguments must resolve to the same C typedef.
///
/// [`rustdoc_ir::CanonicalType`] already normalises type paths and unassigned
/// generic type parameters, but it preserves the distinction between
/// `'static` and other lifetimes. This wrapper layers the additional erasure
/// required for C output.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CCanonicalType(Type);

impl CCanonicalType {
    pub fn new(ct: CanonicalType) -> Self {
        Self(erase_lifetimes(ct.into_inner()))
    }

    pub fn inner(&self) -> &Type {
        &self.0
    }
}

fn erase_lifetimes(ty: Type) -> Type {
    match ty {
        Type::Path(p) => Type::Path(erase_path(p)),
        Type::TypeAlias(p) => Type::TypeAlias(erase_path(p)),
        Type::Reference(r) => Type::Reference(TypeReference {
            is_mutable: r.is_mutable,
            lifetime: Lifetime::Elided,
            inner: Box::new(erase_lifetimes(*r.inner)),
        }),
        Type::Tuple(t) => Type::Tuple(Tuple {
            elements: t.elements.into_iter().map(erase_lifetimes).collect(),
        }),
        Type::Slice(s) => Type::Slice(Slice {
            element_type: Box::new(erase_lifetimes(*s.element_type)),
        }),
        Type::Array(a) => Type::Array(Array {
            element_type: Box::new(erase_lifetimes(*a.element_type)),
            len: a.len,
        }),
        Type::RawPointer(r) => Type::RawPointer(RawPointer {
            is_mutable: r.is_mutable,
            inner: Box::new(erase_lifetimes(*r.inner)),
        }),
        Type::FunctionPointer(fp) => Type::FunctionPointer(FunctionPointer {
            inputs: fp
                .inputs
                .into_iter()
                .map(|input| FunctionPointerInput {
                    name: input.name,
                    type_: erase_lifetimes(input.type_),
                })
                .collect(),
            output: fp.output.map(|t| Box::new(erase_lifetimes(*t))),
            abi: fp.abi,
            is_unsafe: fp.is_unsafe,
        }),
        Type::ScalarPrimitive(_) | Type::Generic(_) => ty,
    }
}

fn erase_path(p: PathType) -> PathType {
    let generic_arguments = p
        .generic_arguments
        .into_iter()
        .map(|arg| match arg {
            GenericArgument::TypeParameter(t) => GenericArgument::TypeParameter(erase_lifetimes(t)),
            GenericArgument::Lifetime(_) => {
                GenericArgument::Lifetime(GenericLifetimeParameter::Inferred)
            }
            GenericArgument::Const(c) => GenericArgument::Const(c),
        })
        .collect();
    PathType {
        package_id: p.package_id,
        rustdoc_id: p.rustdoc_id,
        base_type: p.base_type,
        generic_arguments,
    }
}
