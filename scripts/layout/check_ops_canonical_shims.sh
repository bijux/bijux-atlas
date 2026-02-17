#!/usr/bin/env sh
# Purpose: enforce ops/ as canonical for operational assets and keep root compat shims clean.
# Inputs: root e2e/load/observability paths.
# Outputs: non-zero when compat paths contain real content.
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
errors=0

check_path() {
  name="$1"
  path="$ROOT/$name"

  if [ -L "$path" ]; then
    return 0
  fi

  if [ ! -d "$path" ]; then
    echo "missing compat path: $name" >&2
    errors=1
    return 0
  fi

  files="$(find "$path" -mindepth 1 -maxdepth 2 -type f | sed "s#$ROOT/##" | sort)"
  count="$(printf '%s\n' "$files" | sed '/^$/d' | wc -l | tr -d ' ')"
  first="$(printf '%s\n' "$files" | sed '/^$/d' | head -n1)"

  if [ "$count" -ne 1 ] || [ "$first" != "$name/README.md" ]; then
    echo "compat path '$name' must contain only README.md or be a symlink" >&2
    printf '%s\n' "$files" >&2
    errors=1
  fi
}

check_path e2e
check_path load
check_path observability

if [ "$errors" -ne 0 ]; then
  exit 1
fi

echo "ops canonical shim check passed"
