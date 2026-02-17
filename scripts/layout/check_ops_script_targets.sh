#!/usr/bin/env sh
# Purpose: enforce that every executable ops/**/scripts entrypoint is reachable via a make target.
# Inputs: ops/**/scripts/*.sh and makefiles/ops.mk target list.
# Outputs: non-zero exit when any script is not referenced by make target recipes.
set -eu

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)"
OPS_MK="$ROOT/makefiles/ops.mk"

[ -f "$OPS_MK" ] || { echo "missing $OPS_MK" >&2; exit 1; }

missing=0
for script in $(find "$ROOT/ops" -type f -path '*/scripts/*.sh' | sort); do
  rel="${script#$ROOT/}"
  if ! rg -n --fixed-strings "$rel" "$OPS_MK" >/dev/null; then
    echo "ops script not mapped by make target: $rel" >&2
    missing=1
  fi
done

if [ "$missing" -ne 0 ]; then
  exit 1
fi

echo "ops script coverage check passed"
