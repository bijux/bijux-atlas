#!/usr/bin/env bash
# Purpose: fail on forbidden root-level junk files.
# Inputs: repository root filesystem.
# Outputs: non-zero on forbidden files.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
FORBIDDEN_FILES=(.DS_Store Thumbs.db)
fail=0
for name in "${FORBIDDEN_FILES[@]}"; do
  if [ -f "$ROOT/$name" ]; then
    echo "forbidden root file exists: $name" >&2
    fail=1
  fi
done

if [ "$fail" -ne 0 ]; then
  exit 1
fi

echo "forbidden root files check passed"
