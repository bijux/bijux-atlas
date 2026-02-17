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

allowed_entries=(
  "_lib"
  "datasets"
  "e2e"
  "fixtures"
  "k8s"
  "load"
  "observability"
  "stack"
  "tool-versions.json"
  "README.md"
)

for dir in "${required_dirs[@]}"; do
  if [ ! -d "$dir" ]; then
    echo "missing required ops directory: ${dir#$ROOT/}" >&2
    exit 1
  fi
done

while IFS= read -r entry; do
  name="$(basename "$entry")"
  allowed=0
  for allow in "${allowed_entries[@]}"; do
    if [ "$name" = "$allow" ]; then
      allowed=1
      break
    fi
  done
  if [ "$allowed" -ne 1 ]; then
    echo "forbidden: unexpected ops/ root entry: ops/$name" >&2
    exit 1
  fi
done < <(find "$OPS" -mindepth 1 -maxdepth 1 -print | sort)

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
