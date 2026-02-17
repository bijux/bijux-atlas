#!/usr/bin/env sh
# Purpose: enforce ops/ as canonical for operational assets and keep root compat shims clean.
# Inputs: root ops/e2e/ops/load/observability paths.
# Outputs: non-zero when compat paths contain real content.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
errors=0

check_path() {
  name="$1"
  path="$ROOT/$name"

  if [ ! -L "$path" ]; then
    echo "compat path '$name' must be a symlink" >&2
    errors=1
    return 0
  fi
}

check_path e2e
check_path load
check_path observability

if [ "$errors" -ne 0 ]; then
  exit 1
fi

echo "ops canonical shim check passed"
