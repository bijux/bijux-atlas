#!/usr/bin/env bash
# Purpose: enforce canonical ops workspace layout and separation of concerns.
# Inputs: ops directory tree.
# Outputs: non-zero when required directories are missing or e2e contains stack manifests.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
OPS="$ROOT/ops"

required_dirs=(
  "$OPS/stack"
  "$OPS/k8s"
  "$OPS/load"
  "$OPS/observability"
  "$OPS/datasets"
  "$OPS/fixtures"
  "$OPS/_lib"
  "$OPS/e2e"
)

for dir in "${required_dirs[@]}"; do
  if [ ! -d "$dir" ]; then
    echo "missing required ops directory: ${dir#$ROOT/}" >&2
    exit 1
  fi
done

if [ -e "$OPS/e2e/stack" ] && [ ! -L "$OPS/e2e/stack" ]; then
  echo "forbidden: ops/e2e/stack must be a pointer (symlink) to ops/stack" >&2
  exit 1
fi

if find "$OPS/e2e" -type f \( -name '*.yaml' -o -name '*.yml' \) | grep -q .; then
  echo "forbidden: manifest files found under ops/e2e; keep stack manifests under ops/stack" >&2
  find "$OPS/e2e" -type f \( -name '*.yaml' -o -name '*.yml' \) | sed "s|^$ROOT/||" >&2
  exit 1
fi

echo "ops workspace layout check passed"
