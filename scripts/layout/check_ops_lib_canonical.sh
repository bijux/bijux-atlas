#!/usr/bin/env sh
# Purpose: enforce ops/_lib as the canonical shared shell helper location.
# Inputs: ops tree.
# Outputs: non-zero when duplicate common libs are found outside ops/_lib.
set -eu

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

violations=""
while IFS= read -r file; do
  case "$file" in
    ops/_lib/*|ops/k8s/tests/k8s-suite-lib.sh|ops/k8s/tests/k8s-contract-lib.sh|ops/obs/tests/observability-test-lib.sh) ;;
    *) violations="$violations$file
" ;;
  esac
done <<EOF
$(find ops -type f -name 'common.sh' | sed 's#^\./##' | sort)
EOF

if [ -n "$violations" ]; then
  echo "ops/_lib canonical lib policy failed:" >&2
  printf "%s" "$violations" >&2
  exit 1
fi

echo "ops/_lib canonical lib policy passed"
