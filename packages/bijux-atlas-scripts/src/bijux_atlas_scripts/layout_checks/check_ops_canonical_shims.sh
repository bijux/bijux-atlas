#!/usr/bin/env bash
# Purpose: enforce ops-only canonical layout (no legacy root ops aliases).
# Inputs: root legacy names.
# Outputs: non-zero when deprecated root aliases exist.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
errors=0
for name in e2e load observability charts datasets fixtures; do
  if [ -e "$ROOT/$name" ] || [ -L "$ROOT/$name" ]; then
    echo "deprecated root alias exists: $name" >&2
    errors=1
  fi
done

if [ "$errors" -ne 0 ]; then
  exit 1
fi

echo "ops canonical layout check passed"
