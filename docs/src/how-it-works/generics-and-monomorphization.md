# Generics and monomorphization

Rust generics don't exist in C. `Wrapper<i32>` and `Wrapper<f32>` are
treated as _distinct_ C types: they have different field types, different sizes,
different layouts. `cheadergen` resolves this by **monomorphizing** every
concrete instantiation it sees into a separate, concretely-named C type.

This page explains the naming convention, the placement rule, and the
duplicate-definition guards you'll see in partitioned output.

## Naming

A monomorphized instantiation gets a C name built from the generic's name and
the type parameters:

```rust
#[repr(C)]
pub struct Wrapper<T> {
    pub value: T,
}

#[unsafe(no_mangle)]
pub extern "C" fn unwrap_i32(w: Wrapper<i32>) -> i32 { w.value }

#[unsafe(no_mangle)]
pub extern "C" fn unwrap_f32(w: Wrapper<f32>) -> f32 { w.value }
```

```c
typedef struct {
    int32_t value;
} Wrapper_i32;

typedef struct {
    float value;
} Wrapper_f32;

int32_t unwrap_i32(Wrapper_i32 w);
float unwrap_f32(Wrapper_f32 w);
```

For enums, each instantiation produces a small family of auxiliary
types. For example,

```rust
#[repr(C, u8)]
pub enum Either<T> {
    Left(T),
    Right(i32),
}
```

is rendered on the C side as a tagged union via `Either_i32_Tag` (the discriminant)
and `Either_i32` (the outer struct). All members of a family travel together; see ["Placement"](#placement) below.

## Placement

In [bundled mode](../what-you-get/partitioning.md) the question is moot: every
monomorphization ends up in the single output header. In partitioned mode
`cheadergen` has to pick one of (potentially) several headers as the owner.

The rule is: **a monomorphized instantiation lives in the consuming crate's
header**, where "consumer" means the crate whose non-generic type or
`extern "C"` item references the instantiation.

| Scenario                                                                        | Where `Wrapper_i32` lives                               |
| ------------------------------------------------------------------------------- | ------------------------------------------------------- |
| Target `A` has `extern "C" fn foo(w: Wrapper<i32>)`                             | `A.h`                                                   |
| Non-target `B` has `struct BStruct { field: Wrapper<i32> }`, used by target `A` | `B.h` (the _earliest_ consumer in the dependency chain) |
| Both `B` and `A` use `Wrapper<i32>` directly                                    | Both, with `#ifndef` guards (see below)                 |

### Why not the defining crate?

The "obvious" choice (put `Wrapper_i32` in the header of the crate that
defines `Wrapper<T>`) can result in **synthetic includes.**

If `Wrapper<MyStruct>` lived in `wrapper.h`, then `wrapper.h` would need `#include "mystruct.h"` even though
the defining crate has no Rust dependency on `MyStruct`'s crate.

Placing monomorphizations in the consuming crate keeps the include graph
faithful to the Rust dependency graph.

## Duplicate-definition guards

When multiple consumers use the same instantiation, `cheadergen` emits the
definition in _each_ consuming crate's header. To prevent redefinition errors
when those headers are included together, every monomorphized type is wrapped
in `#ifndef` guards:

```c
#ifndef WRAPPER_I32_DEFINED
#define WRAPPER_I32_DEFINED
typedef struct {
    int32_t value;
} Wrapper_i32;
#endif /* WRAPPER_I32_DEFINED */
```

A C preprocessor sees the first definition, sets the guard, and skips all
subsequent copies. Duplicates are harmless.

> **Single-header runs skip the guards.** If only one header is being written
> in this invocation, `cheadergen` omits the `#ifndef` wrappers — they would
> just be noise.

## Non-generic types are not guarded

Non-generic types live in their defining crate's header and appear there
exactly once, so guards aren't needed and would just clutter the output.

## See also

- [Bundled vs partitioned output](../what-you-get/partitioning.md) to understand why partitioned mode needs these rules at all.
