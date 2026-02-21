#!/usr/bin/env bash
# Purpose: ensure stack manifests are canonical under ops/stack (no duplicates in ops/e2e).
# Inputs: repository files under ops/e2e.
# Outputs: non-zero if stack manifests are found under ops/e2e.
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
violations="$(find "$ROOT/ops/e2e" -type f \( -name '*.yaml' -o -name '*.yml' \) | sed "s|$ROOT/||" || true)"
if [ -n "$violations" ]; then
  echo "stack manifest consolidation check failed: manifests found under ops/e2e" >&2
  echo "$violations" >&2
  exit 1
fi
echo "stack manifest consolidation check passed"
