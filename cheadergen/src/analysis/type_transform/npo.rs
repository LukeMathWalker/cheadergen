use std::cell::RefCell;
use std::collections::HashMap;

use rustdoc_ir::{PathType, Type};
use rustdoc_processor::CrateCollection;
use rustdoc_processor::GlobalItemId;
use rustdoc_processor::indexing::NoAnnotations;
use rustdoc_types::{Attribute, AttributeRepr, ItemEnum, ReprKind};

use crate::analysis::type_resolution;

/// Checks whether a user-defined `PathType` is NPO-eligible, i.e. it is a
/// `#[repr(transparent)]` struct whose single non-ZST field is itself
/// NPO-eligible (recursively).
///
/// Results are memoized for the lifetime of the checker.
pub struct NpoEligibilityChecker<'a> {
    collection: &'a CrateCollection<NoAnnotations>,
    cache: RefCell<HashMap<PathType, bool>>,
}

impl<'a> NpoEligibilityChecker<'a> {
    pub fn new(collection: &'a CrateCollection<NoAnnotations>) -> Self {
        Self {
            collection,
            cache: RefCell::new(HashMap::new()),
        }
    }

    /// Check whether a user-defined type (identified by its `PathType`) is NPO-eligible.
    ///
    /// A type is NPO-eligible if it is a `#[repr(transparent)]` struct whose single
    /// non-ZST field is itself NPO-eligible (either a standard NPO type like `Box`,
    /// `NonNull`, reference, or fn pointer, or another `#[repr(transparent)]` wrapper
    /// that is recursively NPO-eligible).
    pub fn is_eligible(&self, path: &PathType) -> bool {
        if let Some(&cached) = self.cache.borrow().get(path) {
            return cached;
        }

        // Insert false to break cycles.
        self.cache.borrow_mut().insert(path.clone(), false);

        let result = self.is_eligible_inner(path);

        self.cache.borrow_mut().insert(path.clone(), result);
        result
    }

    fn is_eligible_inner(&self, path: &PathType) -> bool {
        let Some(id) = &path.rustdoc_id else {
            return false;
        };

        let global_id = GlobalItemId::new(*id, path.package_id.clone());
        let item = self.collection.get_item_by_global_type_id(&global_id);

        let ItemEnum::Struct(struct_def) = &item.inner else {
            return false;
        };

        // Must be #[repr(transparent)].
        let is_transparent = item.attrs.iter().any(|attr| {
            matches!(
                attr,
                Attribute::Repr(AttributeRepr {
                    kind: ReprKind::Transparent,
                    ..
                })
            )
        });
        if !is_transparent {
            return false;
        }

        let generic_bindings = match type_resolution::setup_generic_bindings(
            path.base_type.last().map(String::as_str).unwrap_or("?"),
            &struct_def.generics,
            path,
        ) {
            Ok(bindings) => bindings,
            Err(()) => return false,
        };

        let Some(inner_ty) = type_resolution::resolve_transparent_inner_type(
            struct_def,
            &generic_bindings,
            path,
            self.collection,
        ) else {
            return false;
        };

        // Check if the inner type is a standard NPO type or recursively NPO-eligible.
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
