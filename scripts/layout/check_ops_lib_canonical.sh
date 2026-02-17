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
    ops/_lib/*|ops/e2e/k8s/tests/common.sh|ops/k8s/tests/common.sh|ops/observability/tests/common.sh) ;;
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
