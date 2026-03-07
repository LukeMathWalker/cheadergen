---
name: write-test
description: Write a test to improve code coverage. Uses `cargo llvm-cov` to identify uncovered lines, then writes a focused test to cover them. Use when user says "write test", "improve coverage", "cover lines", or "write-test".
---

# Write Test (Coverage-Guided)

Write a single focused test to improve code coverage, guided by `cargo llvm-cov` output.

## Instructions

### Step 1: Identify uncovered lines

If the user specified a target file, run:

```bash
just uncovered <file>
```

If no file was specified, run `just uncovered` to see all uncovered lines, then pick the highest-impact target (most uncovered lines or most important code path).

IMPORTANT: After the initial scan, always scope `just uncovered` to the target file to keep context small.

### Step 2: Read the source

Read the target file to understand what the uncovered lines do. Focus on understanding the code paths that lead to those lines — what inputs, configurations, or conditions trigger them.

### Step 3: Write one test

Write a single focused test that exercises the uncovered code path. Place it in the appropriate test module or file following existing project conventions.

Only write **one** test per invocation. Keep it minimal and targeted at the specific uncovered lines.

### Step 4: Verify improvement

Run coverage again and compare:

```bash
just uncovered <file>
```

Compare the before and after `just uncovered` output. Confirm that previously uncovered lines are now covered. If the test didn't improve coverage as expected, investigate why and adjust.

## Notes

- The `just uncovered` output is compact (one line per file with collapsed ranges) to minimize token usage. Prefer it over reading raw lcov.info.
- If a line remains uncovered after your test, it may be dead code, error handling for impossible states, or require more complex setup. Note this to the user rather than forcing coverage.
