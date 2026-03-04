# Processing pipeline

1. **Invoke rustdoc-json** — Generate JSON for the target crate (with caching).
2. **Filter FFI items** — Select items with `#[no_mangle]`, `extern "C"` ABI, `#[repr(C)]`, etc.
3. **Compute crate closure** — If types from other crates are referenced, resolve their definitions (recursively).
4. **Build IR** — Convert rustdoc-types items to an intermediate representation for C/C++ emission.
5. **Transform** — Transform standard types (`Option<&T>` → `*const T`), rename.
6. **Emit** — Pretty-print C and/or C++ headers.
