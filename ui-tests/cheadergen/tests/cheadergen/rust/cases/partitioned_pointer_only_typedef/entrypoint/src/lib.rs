//! Target uses `dep::Outer` by-value (so it pulls `dep`'s header in via
//! `#include`) and uses `dep2::Aliased` only behind a pointer.
//!
//! Because the entrypoint never references any `dep2` type by value,
//! `dep2` is not in `by_value_from`. `dep2` is also not pruned to opaque-only
//! (it has `Inner`, a real struct), so `Aliased` does not land in
//! `partitioned.opaque_types` either. The forward-declaration / type-hints
//! pipeline currently misses it, and the entrypoint header renders the
//! `Aliased` parameter as `struct Aliased *` under tag styles.

use partitioned_pointer_only_typedef_dep::Outer;
use partitioned_pointer_only_typedef_dep2::Aliased;

#[unsafe(no_mangle)]
pub extern "C" fn use_outer(o: Outer) -> i32 {
    o.inner.x
}

#[unsafe(no_mangle)]
pub extern "C" fn use_aliased(p: *const Aliased) -> *const Aliased {
    p
}
