#!/usr/bin/env bash
# Purpose: ensure CI workflows invoke repository scripts through make targets.
# Inputs: GitHub workflow files.
# Outputs: exits non-zero on forbidden direct script execution.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"

violations=0
while IFS= read -r line; do
  file="${line%%:*}"
  body="${line#*:}"
  echo "direct script invocation forbidden in workflow: $file:$body" >&2
  violations=$((violations + 1))
done < <(rg -n "run:\\s*\\.?/?(scripts|ops)/" "$ROOT/.github/workflows" || true)

if [ "$violations" -gt 0 ]; then
  exit 1
fi

echo "no direct script runs gate passed"
