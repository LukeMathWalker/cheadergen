use rustdoc_ir::{GenericArgument, PathType, RawPointer, Type};
use rustdoc_processor::{ALLOC_PACKAGE_ID_REPR, CORE_PACKAGE_ID_REPR, STD_PACKAGE_ID_REPR};

use crate::Collection;

use super::type_collection::CTypeKind;
use super::type_resolution;

/// Apply type simplifications to all types within a resolved type kind.
pub fn simplify_kind(kind: &mut CTypeKind, collection: &Collection) {
    match kind {
        CTypeKind::Struct(def) => {
            for field in &mut def.fields {
                simplify_type(&mut field.type_, collection);
            }
        }
        CTypeKind::Union(def) => {
            for field in &mut def.fields {
                simplify_type(&mut field.type_, collection);
            }
        }
        CTypeKind::TaggedUnion(def) => {
            for variant in &mut def.variants {
                if let Some(ref mut body) = variant.body {
                    for field in &mut body.fields {
                        simplify_type(&mut field.type_, collection);
                    }
                }
            }
        }
        CTypeKind::Typedef(def) => {
            simplify_type(&mut def.inner, collection);
        }
        CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion | CTypeKind::FieldlessEnum(_) => {}
    }
}

/// Apply type simplifications to a single type, recursing into inner types.
///
/// Rewrites:
/// - `Box<T>` → `*mut T`
/// - `NonNull<T>` → `*mut T`
/// - `ManuallyDrop<T>` → `T`
/// - `UnsafeCell<T>` → `T`
/// - `MaybeUninit<T>` → `T`
/// - `Option<fn(...)>` → `fn(...)` (function pointers are inherently nullable)
/// - `Option<&T>` → `*const T` (nullable const pointer)
/// - `Option<&mut T>` → `*mut T` (nullable mutable pointer)
/// - `Option<Box<T>>` → `*mut T` (null pointer optimization)
/// - `Option<NonNull<T>>` → `*mut T` (null pointer optimization)
/// - `Option<W>` → `W` when `W` is a `#[repr(transparent)]` wrapper around an
///   NPO-eligible type (null pointer optimization for transparent wrappers)
pub fn simplify_type(ty: &mut Type, collection: &Collection) {
    while try_simplify(ty, collection) {}

    match ty {
        Type::RawPointer(p) => simplify_type(&mut p.inner, collection),
        Type::Reference(r) => simplify_type(&mut r.inner, collection),
        Type::Array(a) => simplify_type(&mut a.element_type, collection),
        Type::FunctionPointer(fp) => {
            for input in &mut fp.inputs {
                simplify_type(&mut input.type_, collection);
            }
            if let Some(output) = &mut fp.output {
                simplify_type(output, collection);
            }
        }
        Type::Tuple(t) => {
            for elem in &mut t.elements {
                simplify_type(elem, collection);
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
fn try_simplify(ty: &mut Type, collection: &Collection) -> bool {
    try_simplify_box(ty)
        || try_simplify_nonnull(ty)
        || try_simplify_manually_drop(ty)
        || try_simplify_unsafe_cell(ty)
        || try_simplify_maybe_uninit(ty)
        || try_simplify_option(ty, collection)
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

/// `ManuallyDrop<T>` → `T`
fn try_simplify_manually_drop(ty: &mut Type) -> bool {
    let path = match ty {
        Type::Path(p) | Type::TypeAlias(p) => p,
        _ => return false,
    };
    if !is_manually_drop(path) {
        return false;
    }

    let arg = path.generic_arguments.pop().unwrap();
    let GenericArgument::TypeParameter(inner_ty) = arg else {
        unreachable!();
    };

    *ty = inner_ty;
    true
}

/// `UnsafeCell<T>` → `T`
fn try_simplify_unsafe_cell(ty: &mut Type) -> bool {
    let path = match ty {
        Type::Path(p) | Type::TypeAlias(p) => p,
        _ => return false,
    };
    if !is_unsafe_cell(path) {
        return false;
    }

    let arg = path.generic_arguments.pop().unwrap();
    let GenericArgument::TypeParameter(inner_ty) = arg else {
        unreachable!();
    };

    *ty = inner_ty;
    true
}

/// `MaybeUninit<T>` → `T`
fn try_simplify_maybe_uninit(ty: &mut Type) -> bool {
    let path = match ty {
        Type::Path(p) | Type::TypeAlias(p) => p,
        _ => return false,
    };
    if !is_maybe_uninit(path) {
        return false;
    }

    let arg = path.generic_arguments.pop().unwrap();
    let GenericArgument::TypeParameter(inner_ty) = arg else {
        unreachable!();
    };

    *ty = inner_ty;
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
/// `Option<W>` → `W` when `W` is NPO-eligible (transparent wrapper)
fn try_simplify_option(ty: &mut Type, collection: &Collection) -> bool {
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
        // Option<fn(...)> → fn(...)
        Type::FunctionPointer(_) => {
            let arg = path.generic_arguments.pop().unwrap();
            let GenericArgument::TypeParameter(fp_ty) = arg else {
                unreachable!();
            };
            *ty = fp_ty;
            true
        }
        // Option<&T> → *const T
        // Option<&mut T> -> *mut T
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
        // Option<Box<T>> → *mut T
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
        // Option<NonNull<T>> → *mut T
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
        // Option<W> → W, if W is NPO-eligible
        Type::Path(inner_path) | Type::TypeAlias(inner_path)
            if is_user_npo_eligible(inner_path, collection) =>
        {
            let arg = path.generic_arguments.pop().unwrap();
            let GenericArgument::TypeParameter(inner_ty) = arg else {
                unreachable!();
            };
            *ty = inner_ty;
            true
        }
        _ => false,
    }
}

/// Check whether a user-defined type (identified by its `PathType`) is NPO-eligible.
///
/// A type is NPO-eligible if it is a `#[repr(transparent)]` struct whose single
/// non-ZST field is itself NPO-eligible (either a standard NPO type like `Box`,
/// `NonNull`, reference, or fn pointer, or another `#[repr(transparent)]` wrapper
/// that is recursively NPO-eligible).
fn is_user_npo_eligible(path: &PathType, collection: &Collection) -> bool {
    // UnsafeCell disables niche optimization, so Option<UnsafeCell<T>> must never
    // be simplified via NPO, regardless of T's NPO-eligibility.
    if is_unsafe_cell(path) {
        return false;
    }
    if is_maybe_uninit(path) {
        return false;
    }

    let Some(inner_ty) = type_resolution::transparent_inner_type_for_path(path, collection) else {
        return false;
    };

    if is_std_npo_eligible(&inner_ty) {
        return true;
    }

    match &inner_ty {
        Type::Path(inner_path) | Type::TypeAlias(inner_path) => {
            is_user_npo_eligible(inner_path, collection)
        }
        _ => false,
    }
}

/// Returns `true` if the given type is one of the standard library types that
/// are inherently NPO-eligible: references, function pointers, `Box`, or `NonNull`.
///
/// This does **not** check user-defined `#[repr(transparent)]` wrappers — use
/// [`is_user_npo_eligible`] for that.
fn is_std_npo_eligible(ty: &Type) -> bool {
    match ty {
        Type::Reference(_) | Type::FunctionPointer(_) => true,
        Type::Path(p) | Type::TypeAlias(p) => is_box(p) || is_nonnull(p),
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

fn is_manually_drop(p: &PathType) -> bool {
    let pkg = p.package_id.repr();
    (pkg == CORE_PACKAGE_ID_REPR || pkg == STD_PACKAGE_ID_REPR)
        && p.base_type.last().map(String::as_str) == Some("ManuallyDrop")
        && p.generic_arguments.len() == 1
        && matches!(&p.generic_arguments[0], GenericArgument::TypeParameter(_))
}

fn is_unsafe_cell(p: &PathType) -> bool {
    let pkg = p.package_id.repr();
    (pkg == CORE_PACKAGE_ID_REPR || pkg == STD_PACKAGE_ID_REPR)
        && p.base_type.last().map(String::as_str) == Some("UnsafeCell")
        && p.generic_arguments.len() == 1
        && matches!(&p.generic_arguments[0], GenericArgument::TypeParameter(_))
}

fn is_maybe_uninit(p: &PathType) -> bool {
    let pkg = p.package_id.repr();
    (pkg == CORE_PACKAGE_ID_REPR || pkg == STD_PACKAGE_ID_REPR)
        && p.base_type.last().map(String::as_str) == Some("MaybeUninit")
        && p.generic_arguments.len() == 1
        && matches!(&p.generic_arguments[0], GenericArgument::TypeParameter(_))
}
