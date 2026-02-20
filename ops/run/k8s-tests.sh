#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="${RUN_ID:-$OPS_RUN_ID}"
export ARTIFACT_DIR="${ARTIFACT_DIR:-$OPS_RUN_DIR}"
ops_env_load
ops_entrypoint_start "ops-k8s-tests"
ops_version_guard python3 kubectl helm

run_id="${RUN_ID:-${ATLAS_RUN_ID:-k8s-tests-$(date -u +%Y%m%d-%H%M%S)}}"
suite="${ATLAS_E2E_SUITE:-${1:-full}}"
if [ "${1:-}" = "--suite" ]; then
  suite="${2:-full}"
  shift 2
elif [ -n "${1:-}" ] && [[ "${1:-}" == -* ]]; then
  :
elif [ -n "${1:-}" ]; then
  shift 1
fi

out_root="${ATLAS_E2E_OUT_ROOT:-artifacts/evidence/k8s/${run_id}}"
mkdir -p "$out_root"

export RUN_ID="$run_id"
export ATLAS_RUN_ID="$run_id"
export ATLAS_E2E_JSON_OUT="${ATLAS_E2E_JSON_OUT:-$out_root/test-results.json}"
export ATLAS_E2E_JUNIT_OUT="${ATLAS_E2E_JUNIT_OUT:-$out_root/test-results.xml}"
export ATLAS_E2E_SUMMARY_OUT="${ATLAS_E2E_SUMMARY_OUT:-$out_root/test-summary.md}"
export ATLAS_E2E_DEGRADATION_SCORE_OUT="${ATLAS_E2E_DEGRADATION_SCORE_OUT:-$out_root/graceful-degradation-score.json}"
export ATLAS_E2E_SUITE_REPORT="${ATLAS_E2E_JSON_OUT}"

exec "$ROOT/ops/k8s/tests/suite.sh" --suite "$suite" "$@"
