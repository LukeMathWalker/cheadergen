//! References each forward-declarable kind from `dep` only behind a
//! pointer. The dep keeps its own header.
//!
//! The entrypoint header should declare each referenced type inline,
//! without `#include`-ing the dep header:
//!
//! - `union FooUnion;` — plain union forward declaration.
//! - `struct TaggedC;` — data-bearing enum with `#[repr(C)]` is emitted as
//!   a struct, so its forward decl uses the `struct` tag.
//! - `union TaggedInt;` — data-bearing enum with `#[repr(u8)]` is emitted
//!   as a union, so its forward decl uses the `union` tag.
//! - `enum Fieldless { ... }` — fieldless enums cannot be forward-declared
//!   in C; the full definition is emitted inline.

use partitioned_pointer_only_kinds_dep::{Fieldless, FooUnion, TaggedC, TaggedInt};

#[unsafe(no_mangle)]
pub extern "C" fn use_union(p: *const FooUnion) -> *const FooUnion {
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn use_tagged_c(p: *const TaggedC) -> *const TaggedC {
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn use_tagged_int(p: *const TaggedInt) -> *const TaggedInt {
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn use_fieldless(p: *const Fieldless) -> *const Fieldless {
    p
}
