//! References three kinds from `dep` only behind pointers: a typedef, a
//! plain `#[repr(C)]` struct, and a renamed struct. The dep keeps its own
//! header (it has by-value extern fns of its own).
//!
//! The entrypoint header should declare each referenced type inline,
//! without `#include`-ing the dep header:
//!
//! - `typedef uint32_t Aliased;` — typedefs cannot be forward-declared in
//!   C, so the full definition is emitted.
//! - `struct OtherStruct;` — compound forward declaration.
//! - `struct RenamedFoo;` — compound forward declaration using the renamed
//!   C name (the original Rust name `InternalFoo` must not appear).
//!
//! Types not referenced from the entrypoint (e.g. `Anchor`) must not appear
//! in the entrypoint header.

use partitioned_pointer_only_with_header_dep::{Aliased, InternalFoo, OtherStruct};

#[unsafe(no_mangle)]
pub extern "C" fn use_aliased(p: *const Aliased) -> *const Aliased {
    p
}

#[unsafe(no_mangle)]
pub extern "C" fn use_other(o: *const OtherStruct) -> *const OtherStruct {
    o
}

#[unsafe(no_mangle)]
pub extern "C" fn use_foo(f: *const InternalFoo) -> *const InternalFoo {
    f
}
