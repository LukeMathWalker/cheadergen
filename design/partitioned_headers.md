# Partitioned Headers

## Problem

Today, each target package gets a standalone header: if both package A and package B need a
definition of `MyConfig`, it gets duplicated in each header. The generated headers don't
compose — they're independent artifacts that happen to share types.

## Goal

Generated headers should **work together**. Types are partitioned by defining crate: if package A
uses `BType` from package B, A's header `#include`s B's generated header rather than inlining
`BType`'s definition.

This includes generating headers for crates that are not in the "target" group but provide types
used by target packages (e.g. another workspace member or a dependency fetched from crates.io).

## Design Decisions

### One header per crate, demand-driven

Every crate that provides types to target packages gets its own header. Non-target crates produce
"types-only" headers (no `extern "C"` function declarations).

The included types are **demand-driven**: only types actually required by target packages are emitted.
If crate B defines `BType1`, `BType2`, and `BType3`, but targets only use `BType1` and `BType2`,
then B's header contains only those two.

### Include vs forward-declare: per-crate granularity

The decision to `#include` or forward-declare is made **per dependency crate**, not per type:

- If **any** type from crate B is used **by-value** in crate A's header, then A emits
  `#include "b.h"`. No point in including _and_ forward-declaring from the same crate.
- Forward declarations are only used when **all** types from a given dependency crate are used
  exclusively behind pointers.

### Opaque packages: inline forward declarations

When a package is configured as `types = "opaque"`, its types are forward-declared directly in the
consuming header. No separate header file is generated for opaque packages. This keeps the file
count down — forward declarations (`struct Foo;`) are trivially safe to duplicate.

If a user wants to provide their own header for a dependency, the `types = "skip"` mechanism
covers that: skip generation entirely, add the include manually.

### Include paths: bare filenames

Generated `#include` directives use bare filenames: `#include "b.h"`. All generated headers are
assumed to live in the same output directory.

### Backward compatibility: `--bundle` flag

Partitioned output is the **default** for all invocations. When there is exactly one target package,
users can pass `--bundle` to produce a single combined header (matching today's behavior). `--bundle`
is invalid when multiple target packages are selected.

## Generic Types

### Background

Rust generics don't exist in C. cheadergen monomorphizes each concrete instantiation into a
separate C type: `Wrapper<i32>` becomes `Wrapper_i32`, `Either<f32>` becomes `Either_f32`, etc.
Tagged unions produce auxiliary types too: `Either_i32_Tag` (the tag enum) and the main
`Either_i32` struct.

The question for partitioned headers: **which header file owns a monomorphized instantiation?**

### The problem with "defining crate owns instantiations"

The natural instinct is to put all `Wrapper_*` types in the header of the crate that defines
`Wrapper<T>`. This keeps instantiations grouped and avoids duplication. But it can create
**include dependencies that don't exist in the Rust crate graph**:

```
Crate B defines Wrapper<T>
Crate C defines MyStruct
Crate A (target) uses Wrapper<MyStruct> in an extern "C" fn
```

If `Wrapper_MyStruct` goes in B's header, then B's header needs `#include "c.h"` for the
definition of `MyStruct` — even though B has no Rust dependency on C.

Worse, this can create **circular includes**:

```
Crate B defines GenericB<T>
Crate C defines GenericC<T>, CType
Crate B also defines BType
Crate A (target) uses GenericB<CType> and GenericC<BType>
```

Placing `GenericB_CType` in B's header requires B → C. Placing `GenericC_BType` in C's header
requires C → B. Circular.

### Rule: monomorphized generics go in the consuming crate's header

A monomorphized instantiation is placed in the header of the crate that **consumes** it — the
crate whose non-generic types or `extern "C"` functions reference the instantiation:

- Target A has `extern "C" fn foo(w: Wrapper<i32>)` → `Wrapper_i32` goes in A's header.
- Non-target B has `struct BStruct { field: Wrapper<i32> }`, target A uses `BStruct` →
  `Wrapper_i32` goes in B's header (B's non-generic type consumes it).
- If multiple crates in the dependency chain consume the same instantiation, it goes in the
  **earliest** (deepest dependency) to avoid duplication.

This rule:

- Avoids synthetic cross-crate include dependencies.
- Avoids circular includes entirely.
- Matches C conventions — there are no "generic definitions" in C, only concrete types that
  belong to whoever needs them.

### Tagged unions (generic enums)

The same rule applies. A monomorphized tagged union like `Either<i32>` produces multiple C
definitions (`Either_i32_Tag`, `Either_i32`). All auxiliary types stay together with the main
definition in the consuming crate's header.

### Each crate emits what it needs, guarded by `#ifndef`

Each crate emits all monomorphized instantiations it directly uses — in function signatures,
struct fields, etc. — without checking what's already provided by included headers.

To prevent C redefinition errors when the same instantiation appears in multiple headers (e.g.
both B's and A's header define `Wrapper_i32`), monomorphized generic types are wrapped in
include guards:

```c
#ifndef WRAPPER_I32_DEFINED
#define WRAPPER_I32_DEFINED
typedef struct {
  int32_t value;
} Wrapper_i32;
#endif /* WRAPPER_I32_DEFINED */
```

This keeps codegen **fully local** — each crate decides what to emit based solely on its own
needs, with no knowledge of what other headers contain. The guards make duplicate definitions
harmless.

Non-generic types don't need guards: they're partitioned by defining crate, so each definition
appears in exactly one header.

## Test Plan

### New multi-crate test cases

Each test case follows the existing pattern: a main crate with `test.toml` and a dependency
crate with all variants excluded.

#### `partitioned_basic`

Target A depends on crate B. B defines a `repr(C)` struct used by-value in A's `extern "C"` fn.

**Verifies:**

- A's header contains `#include "b.h"` and does **not** inline B's struct definition.
- B gets a types-only header with the struct definition and no function declarations.

#### `partitioned_pointer_only`

Target A uses `*const DepStruct` from crate B — pointer only, never by-value.

**Verifies:**

- A's header contains a forward declaration (`typedef struct DepStruct DepStruct;`), no
  `#include "b.h"`.
- No header is generated for B.

#### `partitioned_mixed_usage`

Target A uses `DepType1` by-value and `*const DepType2` behind a pointer, both from crate B.

**Verifies:**

- A's header contains `#include "b.h"` (triggered by `DepType1`).
- `DepType2` is **not** forward-declared in A (it's available via the include).
- B's header contains definitions for both `DepType1` and `DepType2`.

#### `partitioned_transitive`

Target A uses `BStruct` from crate B. `BStruct` has a field of type `CStruct` from crate C.

**Verifies:**

- Three headers generated: A, B, C.
- A's header includes B's. B's header includes C's.
- Each struct is defined exactly once, in its defining crate's header.

#### `partitioned_generic`

Crate B defines `Wrapper<T>`. Target A uses `Wrapper<i32>` and `Wrapper<f32>` in `extern "C"`
functions.

**Verifies:**

- `Wrapper_i32` and `Wrapper_f32` are defined in A's header (the consumer), not B's.
- No header is generated for B (it contributes no non-generic types).

#### `partitioned_generic_enum`

Crate B defines `Either<T>` (a `repr(C)` tagged union). Target A uses `Either<i32>`.

**Verifies:**

- `Either_i32_Tag` and `Either_i32` are both in A's header.
- Auxiliary types stay together with the main type.

#### `partitioned_generic_with_dep_type`

Crate B defines `Wrapper<T>`. Crate C defines `MyStruct`. Target A uses `Wrapper<MyStruct>`.

**Verifies:**

- `Wrapper_MyStruct` goes in A's header (consumer).
- A's header includes C's header (for the `MyStruct` definition used in `Wrapper_MyStruct`'s
  fields).
- No header is generated for B (only provides the generic, no concrete types).

#### `partitioned_generic_in_dep_struct`

Crate C defines `Wrapper<T>`. Crate B defines `BStruct { field: Wrapper<i32> }`. Target A uses
`BStruct`.

**Verifies:**

- `Wrapper_i32` goes in B's header (B is the earliest consumer).
- `BStruct` also in B's header.
- A's header includes B's header only.

#### `partitioned_generic_overlap`

Crate C defines `Wrapper<T>`. Crate B defines `BStruct { field: Wrapper<i32> }`. Target A uses
both `BStruct` and `Wrapper<i32>` directly in function signatures.

**Verifies:**

- `Wrapper_i32` appears in **both** B's and A's headers, each wrapped in `#ifndef` guards.
- A includes B (for `BStruct`). The duplicate `Wrapper_i32` definition is harmless.
- Compiles without redefinition errors.

#### `partitioned_bundle`

Single target A with a dependency B used by-value. Invoked with `--bundle`.

**Verifies:**

- Produces a single header with all types inlined (today's behavior).
- No separate header for B.

#### `partitioned_opaque_unchanged`

Target A with `[package.dep] types = "opaque"`. Same as current `opaque_dependency` test but
run through the partitioned pipeline.

**Verifies:**

- Opaque types are forward-declared inline in A's header.
- No header generated for the opaque dependency.
- Behavior unchanged from before.
