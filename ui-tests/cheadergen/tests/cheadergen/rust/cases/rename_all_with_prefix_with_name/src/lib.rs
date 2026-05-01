//! `rename_all` and `prefix_with_name` compose: the prefix (derived from the
//! enum name) is also subjected to the casing rule, producing a consistently
//! cased C identifier. An explicit `rename = "..."` short-circuits the prefix
//! casing — the explicit C name is used verbatim as the prefix.

/// Variants emitted as `MY_STATUS_OK`, `MY_STATUS_ERROR_STATE` (prefix is also cased).
#[cheadergen::config(export, prefix_with_name, rename_all = "SCREAMING_SNAKE_CASE")]
#[repr(C)]
pub enum MyStatus {
    Ok,
    ErrorState,
}

/// Variants emitted as `Foo_ok`, `Foo_error_state` — explicit `rename` becomes
/// the prefix verbatim; only variants are cased.
#[cheadergen::config(export, rename = "Foo", prefix_with_name, rename_all = "snake_case")]
#[repr(C)]
pub enum Other {
    Ok,
    ErrorState,
}
