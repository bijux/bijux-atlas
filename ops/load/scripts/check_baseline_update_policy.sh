#!/usr/bin/env bash
set -euo pipefail
if git diff --name-only --cached | grep -q '^configs/ops/perf/baselines/'; then
  if [ "${PERF_BASELINE_UPDATE_FLOW:-0}" != "1" ]; then
    echo "baseline update must go through make perf/baseline-update (PERF_BASELINE_UPDATE_FLOW=1 missing)" >&2
    exit 1
  fi
  if [ "${ATLAS_BASELINE_APPROVED:-0}" != "1" ]; then
    echo "baseline update requires explicit approval: set ATLAS_BASELINE_APPROVED=1" >&2
    exit 1
  fi
fi
echo "baseline policy check passed"
