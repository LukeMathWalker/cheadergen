//! Wraps `dep2::Inner` so the entrypoint can pull `dep2`'s header in
//! transitively without referencing `dep2::Aliased` by-value.

use partitioned_pointer_only_typedef_dep2::Inner;

#[repr(C)]
pub struct Outer {
    pub inner: Inner,
}
