//! A type alias annotated with `#[cheadergen::config(export)]` must be emitted
//! as a `typedef` even when no FFI function references it.

/// Alias to a primitive, forced into the header via annotation.
#[cheadergen::config(export)]
pub type Flags = u32;

#[unsafe(no_mangle)]
pub extern "C" fn noop() {}
