#!/usr/bin/env bash
set -euo pipefail
if git diff --name-only --cached | grep -q '^ops/load/baselines/'; then
  if [ "${ATLAS_BASELINE_APPROVED:-0}" != "1" ]; then
    echo "baseline update requires explicit approval: set ATLAS_BASELINE_APPROVED=1" >&2
    exit 1
  fi
fi
echo "baseline policy check passed"
