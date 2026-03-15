use rustdoc_ir::{GenericArgument, PathType, RawPointer, Type};
use rustdoc_processor::{ALLOC_PACKAGE_ID_REPR, CORE_PACKAGE_ID_REPR, STD_PACKAGE_ID_REPR};

use super::type_collection::CTypeKind;

/// Apply type simplifications to all types within a resolved type kind.
pub fn simplify_kind(kind: &mut CTypeKind) {
    match kind {
        CTypeKind::Struct(def) => {
            for field in &mut def.fields {
                simplify_type(&mut field.type_);
            }
        }
        CTypeKind::Union(def) => {
            for field in &mut def.fields {
                simplify_type(&mut field.type_);
            }
        }
        CTypeKind::TaggedUnion(def) => {
            for variant in &mut def.variants {
                if let Some(ref mut body) = variant.body {
                    for field in &mut body.fields {
                        simplify_type(&mut field.type_);
                    }
                }
            }
        }
        CTypeKind::Typedef(def) => {
            simplify_type(&mut def.inner);
        }
        CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion | CTypeKind::FieldlessEnum(_) => {}
    }
}

/// Apply type simplifications to a single type, recursing into inner types.
///
/// Rewrites:
/// - `Box<T>` → `*mut T`
/// - `NonNull<T>` → `*mut T`
/// - `Option<fn(...)>` → `fn(...)` (function pointers are inherently nullable)
/// - `Option<&T>` → `*const T` (nullable const pointer)
/// - `Option<&mut T>` → `*mut T` (nullable mutable pointer)
/// - `Option<Box<T>>` → `*mut T` (null pointer optimization)
/// - `Option<NonNull<T>>` → `*mut T` (null pointer optimization)
pub fn simplify_type(ty: &mut Type) {
    while try_simplify(ty) {}

    match ty {
        Type::RawPointer(p) => simplify_type(&mut p.inner),
        Type::Reference(r) => simplify_type(&mut r.inner),
        Type::Array(a) => simplify_type(&mut a.element_type),
        Type::FunctionPointer(fp) => {
            for input in &mut fp.inputs {
                simplify_type(&mut input.type_);
            }
            if let Some(output) = &mut fp.output {
                simplify_type(output);
            }
        }
        Type::Tuple(t) => {
            for elem in &mut t.elements {
                simplify_type(elem);
            }
        }
        Type::Path(_)
        | Type::TypeAlias(_)
        | Type::ScalarPrimitive(_)
        | Type::Generic(_)
        | Type::Slice(_) => {}
    }
}

/// Transforms `ty` if any of the known rules apply.
///
/// Returns `true` if `ty` was modified.
fn try_simplify(ty: &mut Type) -> bool {
    try_simplify_box(ty) || try_simplify_nonnull(ty) || try_simplify_option(ty)
}

/// `Box<T>` → `*mut T`
fn try_simplify_box(ty: &mut Type) -> bool {
    let path = match ty {
        Type::Path(p) | Type::TypeAlias(p) => p,
        _ => return false,
    };
    if !is_box(path) {
        return false;
    }

    let arg = path.generic_arguments.pop().unwrap();
    let GenericArgument::TypeParameter(inner_ty) = arg else {
        unreachable!();
    };

    *ty = Type::RawPointer(RawPointer {
        is_mutable: true,
        inner: Box::new(inner_ty),
    });
    true
}

/// `NonNull<T>` → `*mut T`
fn try_simplify_nonnull(ty: &mut Type) -> bool {
    let path = match ty {
        Type::Path(p) | Type::TypeAlias(p) => p,
        _ => return false,
    };
    if !is_nonnull(path) {
        return false;
    }

    let arg = path.generic_arguments.pop().unwrap();
    let GenericArgument::TypeParameter(inner_ty) = arg else {
        unreachable!();
    };

    *ty = Type::RawPointer(RawPointer {
        is_mutable: true,
        inner: Box::new(inner_ty),
    });
    true
}

/// `Option<fn(...)>` → `fn(...)`
/// `Option<&T>` → `*const T`
/// `Option<&mut T>` → `*mut T`
/// `Option<Box<T>>` → `*mut T`
/// `Option<NonNull<T>>` → `*mut T`
fn try_simplify_option(ty: &mut Type) -> bool {
    let path = match ty {
        Type::Path(p) | Type::TypeAlias(p) => p,
        _ => return false,
    };
    if !is_option(path) {
        return false;
    }

    let GenericArgument::TypeParameter(ref inner) = path.generic_arguments[0] else {
        return false;
    };

    match inner {
        Type::FunctionPointer(_) => {
            let arg = path.generic_arguments.pop().unwrap();
            let GenericArgument::TypeParameter(fp_ty) = arg else {
                unreachable!();
            };
            *ty = fp_ty;
            true
        }
        Type::Reference(r) => {
            let is_mutable = r.is_mutable;
            let arg = path.generic_arguments.pop().unwrap();
            let GenericArgument::TypeParameter(Type::Reference(r)) = arg else {
                unreachable!();
            };
            *ty = Type::RawPointer(RawPointer {
                is_mutable,
                inner: r.inner,
            });
            true
        }
        Type::Path(inner_path) | Type::TypeAlias(inner_path) if is_box(inner_path) => {
            // Pop Option's generic arg to get the Box path
            let arg = path.generic_arguments.pop().unwrap();
            let GenericArgument::TypeParameter(inner_ty) = arg else {
                unreachable!();
            };
            // Now extract T from Box<T>
            let inner_path = match inner_ty {
                Type::Path(p) | Type::TypeAlias(p) => p,
                _ => unreachable!(),
            };
            let GenericArgument::TypeParameter(t) =
                inner_path.generic_arguments.into_iter().next().unwrap()
            else {
                unreachable!();
            };

            *ty = Type::RawPointer(RawPointer {
                is_mutable: true,
                inner: Box::new(t),
            });
            true
        }
        Type::Path(inner_path) | Type::TypeAlias(inner_path) if is_nonnull(inner_path) => {
            let arg = path.generic_arguments.pop().unwrap();
            let GenericArgument::TypeParameter(inner_ty) = arg else {
                unreachable!();
            };
            let inner_path = match inner_ty {
                Type::Path(p) | Type::TypeAlias(p) => p,
                _ => unreachable!(),
            };
            let GenericArgument::TypeParameter(t) =
                inner_path.generic_arguments.into_iter().next().unwrap()
            else {
                unreachable!();
            };

            *ty = Type::RawPointer(RawPointer {
                is_mutable: true,
                inner: Box::new(t),
            });
            true
        }
        _ => false,
    }
}

fn is_option(p: &PathType) -> bool {
    let pkg = p.package_id.repr();
    (pkg == CORE_PACKAGE_ID_REPR || pkg == STD_PACKAGE_ID_REPR)
        && p.base_type.last().map(String::as_str) == Some("Option")
        && p.generic_arguments.len() == 1
}

fn is_box(p: &PathType) -> bool {
    let pkg = p.package_id.repr();
    pkg == ALLOC_PACKAGE_ID_REPR
        && p.base_type.last().map(String::as_str) == Some("Box")
        && p.generic_arguments.len() == 1
        && matches!(&p.generic_arguments[0], GenericArgument::TypeParameter(_))
}

fn is_nonnull(p: &PathType) -> bool {
    let pkg = p.package_id.repr();
    (pkg == CORE_PACKAGE_ID_REPR || pkg == STD_PACKAGE_ID_REPR)
        && p.base_type.last().map(String::as_str) == Some("NonNull")
        && p.generic_arguments.len() == 1
        && matches!(&p.generic_arguments[0], GenericArgument::TypeParameter(_))
}
