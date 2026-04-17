//! Regression guard: when a type alias lives in a `types = "skip"` package,
//! the alias name must survive in the emitted C output instead of being
//! resolved through to the underlying primitive (`uintptr_t`).

#[unsafe(no_mangle)]
pub extern "C" fn len() -> alias_dep::size_t {
    0
}
