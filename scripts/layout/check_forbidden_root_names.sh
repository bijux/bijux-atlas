#!/usr/bin/env bash
# Purpose: fail when forbidden legacy root names exist as dir/symlink/file.
# Inputs: repository root.
# Outputs: non-zero if forbidden names are present.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
FORBIDDEN=(charts e2e load observability datasets fixtures)
fail=0
for name in "${FORBIDDEN[@]}"; do
  if [ -e "$ROOT/$name" ] || [ -L "$ROOT/$name" ]; then
    echo "forbidden root entry exists: $name" >&2
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "forbidden root names gate passed"
