#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

for file in \
  docs/architecture/boundaries.md \
  docs/architecture/effects.md \
  docs/architecture/boundary-maps.md \
  docs/product/immutability-and-aliases.md \
  docs/contracts/compatibility.md \
  docs/contracts/plugin/spec.md \
  docs/contracts/plugin/mode.md \
  docs/_lint/duplicate-topics.md
do
  if [ ! -f "$file" ]; then
    echo "duplicate-topics check failed: missing canonical file $file" >&2
    exit 1
  fi
done

# Legacy topic files should now be pointer stubs (single-truth policy)
for file in docs/reference/store/immutability-guarantee.md docs/reference/registry/latest-release-alias-policy.md docs/reference/compatibility/umbrella-atlas-matrix.md docs/reference/compatibility/bijux-dna-atlas.md; do
  if ! rg -q "Canonical page:" "$file"; then
    echo "duplicate-topics check failed: $file must be a canonical pointer" >&2
    exit 1
  fi
done

# Owner header policy for newly canonical pages.
for file in docs/architecture/boundaries.md docs/architecture/effects.md docs/architecture/boundary-maps.md docs/product/immutability-and-aliases.md docs/contracts/compatibility.md docs/operations/k8s/INDEX.md docs/operations/runbooks/INDEX.md; do
  if ! rg -q "^- Owner:" "$file"; then
    echo "duplicate-topics check failed: missing Owner header in $file" >&2
    exit 1
  fi
done

echo "duplicate-topics check passed"