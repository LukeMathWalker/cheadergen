---
name: rustdoc
description: Look up Rust dependency API docs locally. Use when the user or task requires information about a dependency's API, types, traits, functions, or modules — e.g. "look up guppy's PackageGraph", "what methods does rustdoc_types::Item have", "show me clap's derive API".
---

# Rustdoc Lookup

Consult locally-generated rustdoc HTML to look up API details from dependency crates. Use this whenever you need to know a type's fields, an enum's variants, a trait's methods, or a function's signature to complete a task — don't guess, look it up.

## Instructions

### Step 1: Identify the target

Determine what you need to look up:
- **Crate name** (required) — e.g. `guppy`, `rustdoc-types`, `clap`
- **Item** (optional) — a struct, enum, trait, function, module, or type alias

If the crate name is unclear, check `Cargo.toml` for available dependencies.

### Step 2: Generate docs if needed

Run:

```bash
cargo doc -p <crate> --no-deps
```

This generates docs for just that crate (no transitive deps), which is fast.

**Note:** If `cargo doc` fails because the crate name isn't an exact match, check `Cargo.toml` or `Cargo.lock` for the correct package name.

### Step 3: Find the right doc page

Crate names with hyphens become underscores in the doc path (e.g. `rustdoc-types` → `rustdoc_types`).

Use this file structure under `target/doc/<crate_name>/`:

| Item kind    | Path                                    |
|--------------|-----------------------------------------|
| Crate root   | `index.html`                            |
| Struct       | `struct.<Name>.html`                    |
| Enum         | `enum.<Name>.html`                      |
| Trait        | `trait.<Name>.html`                     |
| Function     | `fn.<Name>.html`                        |
| Type alias   | `type.<Name>.html`                      |
| Constant     | `constant.<Name>.html`                  |
| Macro        | `macro.<Name>.html`                     |
| Module       | `<module>/index.html`                   |
| All items    | `all.html`                              |

**If the exact item is unknown**, use one of these discovery approaches:
1. Glob for matching files: `target/doc/<crate_name>/struct.*.html`
2. Read `all.html` for a full listing of all items in the crate

**If the item is inside a submodule**, the path nests: `target/doc/<crate_name>/<module>/struct.<Name>.html`

**Re-exported items (same crate):** Rustdoc generates the doc page at the *re-export* location, not the original definition site. For example, if a crate re-exports `foo::bar::Baz` as `foo::Baz`, the page lives at `struct.Baz.html` in the crate root, not under `bar/`. If a direct path based on the original module doesn't work:
1. Glob for the item name across all subdirectories: `target/doc/<crate_name>/**/struct.<Name>.html`
2. Read `all.html` — it lists every item with its actual doc path regardless of re-export structure

**Re-exported items (from another crate):** With `--no-deps`, items re-exported from a dependency crate won't have doc pages — only a link stub pointing to the source crate's docs, which weren't generated. If the page you need is missing:
1. Identify the source crate (the HTML will contain a cross-crate link like `../other_crate/struct.Name.html`)
2. Generate docs for the source crate too: `cargo doc -p <source_crate> --no-deps`
3. Then read the page at `target/doc/<source_crate>/struct.<Name>.html`

### Step 4: Read and extract what you need

Read the HTML doc page using the Read tool. Rustdoc HTML is structured enough to extract:
- Type/function signatures
- Doc comments and examples
- Method listings (for structs/enums/traits)
- Variant listings (for enums)
- Trait implementations

Extract only the information relevant to the task at hand, then proceed with your work.

## Tips

- For large pages, use the `offset` and `limit` parameters on Read to focus on the relevant section.
- When exploring an unfamiliar crate, start with `index.html` or `all.html` and narrow down.
