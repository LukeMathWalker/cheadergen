# Behavioural differences: cbindgen vs cheadergen

This document tracks known differences in the C header output between
[cbindgen](https://github.com/mozilla/cbindgen) and cheadergen.

## Constants in array lengths

cbindgen parses Rust source directly and preserves named constants in
array-length positions. For example, given:

```rust
pub const FOO: usize = 10;

#[repr(C)]
struct Foo {
    x: [i32; FOO],
}
```

cbindgen emits:

```c
#define FOO 10
// ...
typedef struct {
  int32_t x[FOO];
} Foo;
```

cheadergen uses rustdoc JSON, which resolves constant expressions to their
literal values. The same Rust code produces `"len": "10"` in the JSON, so
cheadergen emits:

```c
typedef struct {
  int32_t x[10];
} Foo;
```

The constant name is lost at the rustdoc JSON level.
