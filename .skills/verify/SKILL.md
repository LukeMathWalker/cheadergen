---
name: verify
description: Runs the cheadergen verification workflow in stages - lint, format, generation tests, and full test suite. Use when user says "verify", "run checks", "run tests", or wants to validate changes before committing.
---

# Verify

Run the project's verification workflow in stages, failing fast on the cheapest checks.

## Instructions

### Step 1: Format

Run

```bash
just fmt
```

to ensure the code is formatted.

### Step 2: Lint

Run the linter, it's the fastest check:

```bash
just lint
```

If `just lint` fails, fix the reported clippy warnings before proceeding.

### Step 3: Header generation tests

Run generation tests only - these skip C/C++ compilation and give fast feedback:

```bash
just test-generate
```

If all tests pass, proceed to Step 4.

#### Handling snapshot mismatches

This project uses insta for snapshot testing. When expected output changes, tests fail and insta writes `.snap.new` files next to the existing `.snap` files.

1. Read the `.snap.new` files to review the diffs and determine if the changes are expected
2. Accept snapshots by moving the `.snap.new` file over the `.snap` file: `mv foo.snap.new foo.snap`
   - To reject, simply delete the `.snap.new` file
3. Re-run `just test-generate` to confirm the updated snapshots pass

IMPORTANT: Do not use `cargo insta` commands (`cargo insta review`, `cargo insta accept`, etc.). Always move `.snap.new` files directly.

### Step 4: Full test suite

Run the full suite if generation tests passed - this includes C/C++ compilation and takes longer:

```bash
just test
```

If compilation tests fail but generation tests passed, the issue is in the generated header output rather than snapshot content.

## Scoping to specific cases

All test commands accept extra nextest filter args:

```bash
# E.g. Run a specific test
just test -E 'test(=cbindgen::generate::c::plain::alias)'
```

## Common Issues

### Nightly toolchain missing

If tests fail with a toolchain error, install nightly:

```bash
rustup toolchain install nightly
```
