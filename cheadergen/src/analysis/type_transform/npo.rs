use rustdoc_ir::{PathType, Type};
use rustdoc_processor::CrateCollection;
use rustdoc_processor::indexing::NoAnnotations;

use crate::analysis::type_resolution;

/// Checks whether a user-defined `PathType` is NPO-eligible, i.e. it is a
/// `#[repr(transparent)]` struct whose single non-ZST field is itself
/// NPO-eligible (recursively).
pub struct NpoEligibilityChecker<'a> {
    collection: &'a CrateCollection<NoAnnotations>,
}

impl<'a> NpoEligibilityChecker<'a> {
    pub fn new(collection: &'a CrateCollection<NoAnnotations>) -> Self {
        Self { collection }
    }

    /// Check whether a user-defined type (identified by its `PathType`) is NPO-eligible.
    ///
    /// A type is NPO-eligible if it is a `#[repr(transparent)]` struct whose single
    /// non-ZST field is itself NPO-eligible (either a standard NPO type like `Box`,
    /// `NonNull`, reference, or fn pointer, or another `#[repr(transparent)]` wrapper
    /// that is recursively NPO-eligible).
    pub fn is_eligible(&self, path: &PathType) -> bool {
        let Some(inner_ty) =
            type_resolution::transparent_inner_type_for_path(path, self.collection)
        else {
            return false;
        };

        if is_std_npo_eligible(&inner_ty) {
            return true;
        }

        match &inner_ty {
            Type::Path(inner_path) | Type::TypeAlias(inner_path) => self.is_eligible(inner_path),
            _ => false,
        }
    }
}

/// Returns `true` if the given type is one of the standard library types that
/// are inherently NPO-eligible: references, function pointers, `Box`, or `NonNull`.
///
/// This does **not** check user-defined `#[repr(transparent)]` wrappers — use
/// [`NpoEligibilityChecker`] for that.
pub(super) fn is_std_npo_eligible(ty: &Type) -> bool {
    match ty {
        Type::Reference(_) | Type::FunctionPointer(_) => true,
        Type::Path(p) | Type::TypeAlias(p) => super::is_box(p) || super::is_nonnull(p),
        _ => false,
    }
}
