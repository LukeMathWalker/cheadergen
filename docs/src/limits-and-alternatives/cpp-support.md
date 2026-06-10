# C++ support

`cheadergen` is, as of today, a **C header generator**. C++ projects can still
consume `cheadergen`'s output, but only as C header(s) included from C++, not
as a native C++ header with C++-idiomatic features.

## What works today: `--cpp-compat`

The CLI flag is `--cpp-compat` (equivalently, `[c].cpp_compat = true` in
`cheadergen.toml`). It tells `cheadergen` to wrap function and static
declarations in an `extern "C"` block guarded by `#ifdef __cplusplus`:

```c
#ifdef __cplusplus
extern "C" {
#endif // __cplusplus

double distance(Point a, Point b);

#ifdef __cplusplus
}  // extern "C"
#endif  // __cplusplus
```

That single addition is enough to make the generated `.h` file safe to
`#include` from C++ source code. The C++ compiler now knows the declarations
inside use the C calling convention and C name mangling (none), so linking
against your Rust library "just works."

The same header is still valid C: the `#ifdef` guards mean a C compiler sees
the declarations directly.

## What's not (yet) supported: native C++ output

`cheadergen` cannot produce `.hpp` files with C++-idiomatic constructs
(e.g. `enum class`, namespaces, templates, destructors, etc.).

Native C++ support is a feature gap and may be addressed in
a future release. There is no scheduled timeline; if you're interested in
it, the
[issue tracker](https://github.com/LukeMathWalker/cheadergen/issues) is the
right place to express the use case or contribute.

In the meantime, `--cpp-compat` is the recommended path for any project that
needs to consume `cheadergen` output from C++.

## See also

- [Limitations](./limitations.md) for a wider view of what `cheadergen` does
  and doesn't do today.
- [Anatomy of a generated header](../what-you-get/header-structure.md) for where the
  `extern "C"` block lands in the generated file.
- [Comparison with `cbindgen`](./vs-cbindgen.md). `cbindgen` does ship some
  C++-idiomatic features that `cheadergen` doesn't.
