#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
PROFILE="${1:-${ATLAS_PERF_BASELINE_PROFILE:-local}}"
RESULTS_DIR="${ATLAS_PERF_RESULTS_DIR:-artifacts/perf/results}"
python3 "$ROOT/ops/load/scripts/update_baseline.py" \
  --profile "$PROFILE" \
  --results "$RESULTS_DIR" \
  --environment "${ATLAS_PERF_ENVIRONMENT:-local}" \
  --k8s-profile "${PROFILE:-kind}" \
  --replicas "${ATLAS_PERF_REPLICAS:-1}"
