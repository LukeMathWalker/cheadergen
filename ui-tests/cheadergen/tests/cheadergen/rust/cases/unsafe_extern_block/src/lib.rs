//! `unsafe extern "C" { ... }` blocks declare symbols that will be resolved
//! at link-time — they are *consumed* by the Rust crate, not *exported* by it.
//!
//! cheadergen must not emit declarations for these functions in the generated
//! header, since our library doesn't define them.

unsafe extern "C" {
    pub fn externally_linked_fn(x: i32) -> i32;

    pub static EXTERNALLY_LINKED_STATIC: i32;
}

#[unsafe(no_mangle)]
pub extern "C" fn exported_fn(x: i32) -> i32 {
    unsafe { externally_linked_fn(x) + EXTERNALLY_LINKED_STATIC }
}
