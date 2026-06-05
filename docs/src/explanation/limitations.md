# Limitations

`cheadergen` is young project: there may be defects that we have yet to uncover.
Furthermore, it has some structural limitations due to the approach it has chosen
for its reflection engine.

It's good to be aware of these limitations ahead of choosing `cheadergen` for
your next project.

## No `#[cfg]` awareness

`cheadergen`'s type analysis uses `rustdoc-json` as its primary source.
`rustdoc` only emits items for the
_current_ compilation target. Items behind a `#[cfg]` predicate that evaluates to `false`
for a given `rustdoc` invocation simply won't appear in the generated JSON:
`cheadergen` won't see them, at all.

Let's look at an example:

```rust
#[cfg(target_os = "linux")]
#[unsafe(no_mangle)]
pub extern "C" fn linux_specific() { /* ... */ }

#[cfg(target_os = "windows")]
#[unsafe(no_mangle)]
pub extern "C" fn windows_specific() { /* ... */ }
```

When you run `cheadergen` on macOS, _neither_ function appears in the generated
header. Run on Linux and only `linux_specific` appears. Run on Windows and
only `windows_specific` appears.

If your bindings need to span multiple platforms today, you can must use a workaround:
**run `cheadergen` once per target platform** and post-process the outputs into a combined header yourself.

It's not great, but it's the best we can suggest at the moment. If your project is `#[cfg]`-heavy, prefer
`cbindgen` for now.

## Constants names may be lost in translation

`cheadergen` may omit named constants when generating C headers.
For example, in this case:

```rust
pub const FOO: usize = 10;

#[repr(C)]
struct Foo {
    x: [i32; FOO],
}
```

`cheadergen` receives
the array length from rustdoc JSON as the literal string `"10"`, so it emits
`int32_t x[10]` and the named `FOO` constant is no longer used to express
the array length.

## No C++ native headers

`cheadergen` can generate a C++-compatible C header via
`cheadergen generate --cpp-compat [...]`, but the output is a C-shaped header
that happens to compile as C++. There are no C++-idiomatic enums, no operator
overloads, no custom constructors. See [C++ support](./cpp-support.md) for
the full picture and the roadmap.

## See also

- [Comparison with `cbindgen`](./vs-cbindgen.md) to determine when to pick one over the
  other.
