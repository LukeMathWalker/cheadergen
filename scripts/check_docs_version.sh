#!/usr/bin/env bash
# Verify that version-pinned references in docs/src/ match their source of
# truth: release URLs must match the workspace package version, and nightly
# toolchain dates must match cheadergen_cli/rust-docs-toolchain. Intended to
# run on release-PR branches so a forgotten docs bump fails CI before the
# release is published.
#
# Usage:
#   scripts/check_docs_version.sh           # auto-detect version from Cargo.toml
#   scripts/check_docs_version.sh 0.3.0     # override expected version

set -euo pipefail

repo_root=$(git rev-parse --show-toplevel)
docs_dir="${repo_root}/docs/src"

if [[ $# -ge 1 ]]; then
    expected="$1"
else
    expected=$(cargo pkgid -p cheadergen_cli --manifest-path "${repo_root}/Cargo.toml" \
        | awk -F'#' '{print $2}')
fi

if [[ -z "${expected}" ]]; then
    echo "error: could not determine expected version from cargo pkgid" >&2
    exit 2
fi

# Match URLs pinned to a release tag: download/X.Y.Z/... or tag/X.Y.Z .
pattern='cheadergen/releases/(download|tag)/[0-9]+\.[0-9]+\.[0-9]+'

mapfile -t matches < <(grep -rEHn "${pattern}" "${docs_dir}" || true)

failed=0

if [[ ${#matches[@]} -eq 0 ]]; then
    echo "OK — no version-pinned release URLs in ${docs_dir#"${repo_root}/"}."
else
    mismatches=()
    for line in "${matches[@]}"; do
        found=$(printf '%s\n' "${line}" \
            | grep -oE "${pattern}" \
            | grep -oE '[0-9]+\.[0-9]+\.[0-9]+' \
            | head -n1)
        if [[ "${found}" != "${expected}" ]]; then
            mismatches+=("${line}")
        fi
    done

    if [[ ${#mismatches[@]} -gt 0 ]]; then
        {
            echo "error: docs reference release version(s) other than ${expected}:"
            printf '  %s\n' "${mismatches[@]}"
            echo
            echo "Bump the URLs in docs/src/ to ${expected} and commit the change."
        } >&2
        failed=1
    else
        echo "OK — all ${#matches[@]} release URL(s) in docs/src/ pin to ${expected}."
    fi
fi

# Nightly toolchain dates in the docs must match the pinned toolchain.
expected_toolchain=$(tr -d '[:space:]' < "${repo_root}/cheadergen_cli/rust-docs-toolchain")
nightly_pattern='nightly-[0-9]{4}-[0-9]{2}-[0-9]{2}'

mapfile -t nightly_matches < <(grep -rEHn "${nightly_pattern}" "${docs_dir}" || true)

nightly_mismatches=()
for line in "${nightly_matches[@]}"; do
    found=$(printf '%s\n' "${line}" | grep -oE "${nightly_pattern}" | head -n1)
    if [[ "${found}" != "${expected_toolchain}" ]]; then
        nightly_mismatches+=("${line}")
    fi
done

if [[ ${#nightly_mismatches[@]} -gt 0 ]]; then
    {
        echo "error: docs reference nightly toolchain(s) other than ${expected_toolchain}:"
        printf '  %s\n' "${nightly_mismatches[@]}"
        echo
        echo "Update the nightly dates in docs/src/ to match cheadergen_cli/rust-docs-toolchain."
    } >&2
    failed=1
elif [[ ${#nightly_matches[@]} -gt 0 ]]; then
    echo "OK — all ${#nightly_matches[@]} nightly toolchain reference(s) in docs/src/ pin to ${expected_toolchain}."
fi

exit "${failed}"
