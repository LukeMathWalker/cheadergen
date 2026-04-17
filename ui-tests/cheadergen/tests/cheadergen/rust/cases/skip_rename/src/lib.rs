//! Verifies that `rename` is honored on items also marked `skip` — the
//! emitted C reference uses the renamed C name, not the Rust identifier.

#[cheadergen::config(skip, rename = "foo_t")]
pub type MyT = u64;

#[unsafe(no_mangle)]
pub extern "C" fn id(x: MyT) -> MyT {
    x
}
