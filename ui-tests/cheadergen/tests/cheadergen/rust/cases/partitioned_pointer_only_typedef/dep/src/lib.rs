//! Wraps `dep2::Inner` so it's reachable by-value through `Outer`. Used to
//! keep `dep2` from being pruned to opaque-only, so the typedef test
//! exercises the pointer-only-with-header path rather than the
//! pruned-opaque path.

use partitioned_pointer_only_typedef_dep2::Inner;

#[repr(C)]
pub struct Outer {
    pub inner: Inner,
}
