//! Verifies that types from a `[package] types = "skip"` crate are emitted
//! by their bare last-segment name in every style — no `struct`/`enum`
//! prefix, no forward declaration, no definition — regardless of whether
//! the Rust side is a type alias, a `#[repr(C)]` struct, or a `#[repr(C)]` enum.

#[unsafe(no_mangle)]
pub extern "C" fn io(
    addr: *const ffi_dep::sockaddr,
    lvl: ffi_dep::log_level,
) -> ffi_dep::size_t {
    let _ = (addr, lvl);
    0
}
