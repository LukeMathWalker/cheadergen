# Commit or regenerate? Managing generated headers

A perennial question for any code generator: should the generated artifacts
live in your repository, or should every consumer rebuild them on demand?

`cheadergen` works fine either way. This guide lays out the two viable
workflows, what each costs, and what each gives you.

## Option 1: commit the generated headers

The headers live in your repo next to the Rust source. A CI check guarantees
they stay in sync.

**Choose this when:**

- You ship a stable C API that downstream consumers depend on.
- Reviewers should see C API changes in pull requests.
- Downstream C consumers don't (and shouldn't have to) install `cheadergen` or a
  Rust nightly.

**The workflow:**

1. Commit the contents of `include/` (or wherever your headers live) to
   version control.
2. Add a CI job that regenerates the headers and fails if `git diff` is
   non-empty:

   ```bash
   cheadergen generate --lang c --output-dir include --prune-orphans .
   git diff --exit-code include/
   ```

   `--prune-orphans` is important: without it, removing a type from the Rust
   source leaves the old header behind, and CI passes when it shouldn't.

3. Document the regeneration command in CONTRIBUTING.md so contributors can
   run it locally before opening a PR.

**Pros:**

- C API changes show up in code review as `+`/`-` lines in real headers.
- Consumers can `git clone` and immediately have a usable C interface.
- No nightly toolchain on the consumer side.
- Works inside restricted CI / build environments that can't install
  `cheadergen`.

**Cons:**

- More churn in PR diffs.
- Easy to forget to regenerate. A CI check to prevent drift is highly recommended.

## Option 2: generate on demand

The headers are a build artifact, never committed. Each consumer runs
`cheadergen` as part of their build.

**Choose this when:**

- The C API is internal to your team and you control all the
  consumers.
- The C API isn't stable yet; reducing PR churn is more valuable than visible
  C API diffs.
- All consumers already have `cheadergen` + a nightly toolchain.

**The workflow:**

- Wire `cheadergen generate` into a command runner (`just`, `make`, etc.) so
  it runs as a step before/after `cargo build`—see
  [Integrate with Cargo](./integrate-with-cargo.md) for the reason this can't
  live inside a `build.rs`.
- Add `include/` (or wherever the generated headers land) to `.gitignore`.

**Pros:**

- Smaller PR diffs.
- No risk of source and generated headers drifting apart, they're regenerated
  every build.

**Cons:**

- Every consumer needs `cheadergen` + nightly Rust on their build machine.
- C API changes aren't visible in code review without extra tooling.
- First-time setup is slower for new contributors.

## Hybrid: commit, but only at release time

A workable middle ground for some libraries: don't commit headers on every
PR, but stamp a regenerated set into the release tarball or GitHub release
artifact at tag time. Contributors don't deal with header diffs, but
downstream consumers still get headers without needing `cheadergen`.

The hook is simply a step in your release workflow that runs
`cheadergen generate ...` and includes the output in the published artifact.
