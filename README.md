# `cheadergen`

`cheadergen` is a ground-up reimplementation of [cbindgen](https://github.com/mozilla/cbindgen)
that uses [**rustdoc-json**](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html) as its
reflection mechanism instead of `syn`-based source parsing.

Status: **alpha quality**, functional.

## Documentation

- Per-item attributes: the [`#[cheadergen::config(...)]`](https://docs.rs/cheadergen/latest/cheadergen/attr.config.html) macro reference.
- `cheadergen.toml`: the [config reference](https://docs.rs/cheadergen/latest/cheadergen/config_reference/index.html).

Until the crate is published, build the same docs locally with `cargo doc --no-deps -p cheadergen --open`.
