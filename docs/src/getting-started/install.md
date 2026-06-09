# Install `cheadergen`

`cheadergen` ships as a single binary called `cheadergen`.
Since `cheadergen` relies on [`rustdoc-json`](https://rust-lang.github.io/rfcs/2963-rustdoc-json.html)
for header generation, it [requires a specific `nightly` toolchain to be installed via `rustup`](#nightly-toolchain).

Pick whichever installation method fits your environment best.

## Prebuilt binaries

On macOS or Linux:

```bash
curl --proto '=https' --tlsv1.2 -LsSf \
  https://github.com/LukeMathWalker/cheadergen/releases/download/0.2.4/cheadergen_cli-installer.sh \
  | sh
```

The installer downloads the prebuilt binary for your platform and drops it into
`~/.cargo/bin` (or, if Cargo isn't installed, `~/.local/bin`).

On Windows, via Powershell:

```powershell
powershell -ExecutionPolicy Bypass -c "irm https://github.com/LukeMathWalker/cheadergen/releases/download/0.2.4/cheadergen_cli-installer.ps1 | iex"
```

Installers download prebuilt binaries from our [GitHub releases](https://github.com/LukeMathWalker/cheadergen/releases/latest). Each archive ships with a sibling `.sha256` file you can use to verify the
download.

## From crates.io

```bash
cargo install --locked cheadergen_cli
```

`cheadergen_cli` is the binary crate; the installed executable is named
`cheadergen`. If you have [`cargo-binstall`](https://github.com/cargo-bins/cargo-binstall)
it will fetch the prebuilt artifacts:

```bash
cargo binstall cheadergen_cli
```

## Nightly toolchain

`cheadergen` invokes `cargo doc --output-format=json`, which is only
available on nightly. You don't need to make nightly your default toolchain:
`cheadergen` calls a pinned nightly via `cargo +<toolchain>`.

If the pinned nightly isn't installed when `cheadergen` runs, header generation
will fail with an error that explains how to install the missing toolchain.

If you want to install it eagerly, run:

```bash
rustup install nightly-2026-06-04 -c rust-docs-json
```

## Verify the install

```bash
cheadergen --help
cheadergen generate --help
```

Both commands should print usage information without errors. You're ready for
the [Quickstart](./quickstart.md).
