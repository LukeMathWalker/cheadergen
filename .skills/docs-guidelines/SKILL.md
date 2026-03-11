---
name: docs-guidelines
description: Guidelines for writing Rust documentation. Refer to this when you need to write doc comments on functions, types or modules.
---

# Docs Guidelines

Standards to follow when writing code-level documentation.

## Guidelines

- Key concepts should be explained only once. All other documentation should use an intra-documentation link to the first explanation.
- Always use an intra-documentation link when mentioning a Rust symbol (type, function, constant, etc.).
- Avoid referring to specific lines or line ranges, as they may change over time.
  Use line comments if the documentation needs to be attached to a specific code section inside
  a function/method body.
- Focus on why, not how.
  In particular, avoid explaining trivial implementation details in line comments.
- Refer to constants using intra-documentation links. Don't hard-code their values in the documentation of other items.
- Intra-documentation links to private items are allowed. Don't duplicate.
