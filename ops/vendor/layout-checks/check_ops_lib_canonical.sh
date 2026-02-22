#!/usr/bin/env bash
# Purpose: enforce atlasctl-owned canonical shell helper assets and retired ops/_lib/common.sh.
# Inputs: ops tree and atlasctl ops assets.
# Outputs: non-zero when retired ops/_lib/common.sh exists or duplicate common libs are found in ops.
set -euo pipefail

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

violations=""
if [ -f "ops/_lib/common.sh" ]; then
  violations="$violations""ops/_lib/common.sh (retired; use packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/lib/ops_common.sh)\n"
fi
while IFS= read -r file; do
  case "$file" in
    ops/k8s/tests/checks/_lib/common.sh|ops/k8s/tests/checks/_lib/k8s-suite-lib.sh|ops/k8s/tests/checks/_lib/k8s-contract-lib.sh|ops/load/tests/common.sh|ops/obs/tests/common.sh|ops/obs/tests/observability-test-lib.sh) ;;
    *) violations="$violations$file
" ;;
  esac
done <<EOF
$(find ops -type f -name 'common.sh' | sed 's#^\./##' | sort)
EOF

if [ -n "$violations" ]; then
  echo "ops shell helper canonicalization policy failed:" >&2
  printf "%s" "$violations" >&2
  exit 1
fi

echo "ops shell helper canonicalization policy passed"
