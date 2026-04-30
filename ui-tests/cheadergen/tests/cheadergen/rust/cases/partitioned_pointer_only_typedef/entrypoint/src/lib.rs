//! Uses `dep::Outer` by-value (so the entrypoint header `#include`s `dep`)
//! and `dep2::Aliased` only behind a pointer.
//!
//! The entrypoint header should declare `Aliased` inline as
//! `typedef uint32_t Aliased;` — it must not `#include` `dep2`'s header
//! directly, even though the typedef lives there. The function signature
//! must spell the parameter as `Aliased`, not `struct Aliased`.

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
