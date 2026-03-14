use std::collections::HashSet;

use rustdoc_ir::{FreeFunction, GenericArgument, PathType, Type};
use rustdoc_processor::{CORE_PACKAGE_ID_REPR, STD_PACKAGE_ID_REPR};

use super::type_collection::{CTypeDefinition, CTypeKind, c_type_name};
use crate::static_item::StaticItem;

/// Apply type simplifications to all types in the IR:
/// - `Option<FunctionPointer>` → `FunctionPointer` (function pointers are inherently nullable)
///
/// Removes type definitions that were simplified away (e.g., `Option`
/// opaques that are no longer referenced).
pub fn simplify_option_fn_ptrs(
    type_defs: &mut Vec<CTypeDefinition>,
    functions: &mut [FreeFunction],
    statics: &mut [StaticItem],
) {
    let mut removed_names: HashSet<String> = HashSet::new();

    for def in type_defs.iter_mut() {
        simplify_kind(&mut def.kind, &mut removed_names);
    }
    for func in functions.iter_mut() {
        for input in &mut func.header.inputs {
            simplify_type(&mut input.type_, &mut removed_names);
        }
        if let Some(output) = &mut func.header.output {
            simplify_type(output, &mut removed_names);
        }
    }
    for s in statics.iter_mut() {
        simplify_type(&mut s.type_, &mut removed_names);
    }

    if !removed_names.is_empty() {
        type_defs.retain(|def| !removed_names.contains(&def.name));
    }
}

fn simplify_kind(kind: &mut CTypeKind, removed: &mut HashSet<String>) {
    match kind {
        CTypeKind::Struct(def) => {
            for field in &mut def.fields {
                simplify_type(&mut field.type_, removed);
            }
        }
        CTypeKind::Union(def) => {
            for field in &mut def.fields {
                simplify_type(&mut field.type_, removed);
            }
        }
        CTypeKind::TaggedUnion(def) => {
            for variant in &mut def.variants {
                if let Some(ref mut body) = variant.body {
                    for field in &mut body.fields {
                        simplify_type(&mut field.type_, removed);
                    }
                }
            }
        }
        CTypeKind::Typedef(def) => {
            simplify_type(&mut def.inner, removed);
        }
        CTypeKind::OpaqueStruct | CTypeKind::OpaqueUnion | CTypeKind::FieldlessEnum(_) => {}
    }
}

fn simplify_type(ty: &mut Type, removed: &mut HashSet<String>) {
    if try_simplify_option_fn_ptr(ty, removed) {
        // Recurse into the replacement (fn ptr inputs/output may also need simplification).
        simplify_type(ty, removed);
        return;
    }

    match ty {
        Type::RawPointer(p) => simplify_type(&mut p.inner, removed),
        Type::Reference(r) => simplify_type(&mut r.inner, removed),
        Type::Array(a) => simplify_type(&mut a.element_type, removed),
        Type::FunctionPointer(fp) => {
            for input in &mut fp.inputs {
                simplify_type(input, removed);
            }
            if let Some(output) = &mut fp.output {
                simplify_type(output, removed);
            }
        }
        Type::Tuple(t) => {
            for elem in &mut t.elements {
                simplify_type(elem, removed);
            }
        }
        Type::Path(_) | Type::TypeAlias(_) | Type::ScalarPrimitive(_) | Type::Generic(_)
        | Type::Slice(_) => {}
    }
}

/// If `ty` is `Option<FunctionPointer>` from core/std, replace it with the
/// inner `FunctionPointer` and record the old type name for cleanup.
fn try_simplify_option_fn_ptr(ty: &mut Type, removed: &mut HashSet<String>) -> bool {
    // Check if this matches Option<FunctionPointer>.
    let is_match = match ty {
        Type::Path(p) | Type::TypeAlias(p) => is_option_wrapping_fn_ptr(p),
        _ => false,
    };

    if !is_match {
        return false;
    }

    // Extract the function pointer.
    let path_type = match ty {
        Type::Path(p) | Type::TypeAlias(p) => p,
        _ => unreachable!(),
    };

    let old_name = c_type_name(&Type::Path(path_type.clone()));
    let arg = path_type.generic_arguments.pop().unwrap();
    let GenericArgument::TypeParameter(fp_ty) = arg else {
        unreachable!();
    };

    removed.insert(old_name);
    *ty = fp_ty;
    true
}

fn is_option_wrapping_fn_ptr(p: &PathType) -> bool {
    let pkg = p.package_id.repr();
    if pkg != CORE_PACKAGE_ID_REPR && pkg != STD_PACKAGE_ID_REPR {
        return false;
    }
    if p.base_type.last().map(String::as_str) != Some("Option") {
        return false;
    }
    if p.generic_arguments.len() != 1 {
        return false;
    }
    matches!(
        &p.generic_arguments[0],
        GenericArgument::TypeParameter(Type::FunctionPointer(_))
    )
}
