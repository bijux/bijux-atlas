#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-stack-up"
ops_version_guard kind kubectl helm

reuse=0
profile="${PROFILE:-kind}"
while [ "$#" -gt 0 ]; do
  case "$1" in
    --reuse) reuse=1 ;;
    --profile)
      profile="${2:-$profile}"
      shift
      ;;
    *)
      echo "usage: ops/run/stack-up.sh [--reuse] [--profile <id>]" >&2
      exit 2
      ;;
  esac
  shift
done

start_ts="$(date +%s)"
status="pass"
log_file="artifacts/evidence/stack/${RUN_ID}/stack-up.log"
health_json="artifacts/evidence/stack/${RUN_ID}/health-report.json"
snapshot_json="artifacts/evidence/stack/state-snapshot.json"
atlas_ns="${ATLAS_E2E_NAMESPACE:?ATLAS_E2E_NAMESPACE is required by configs/ops/env.schema.json}"
atlas_cluster="${ATLAS_E2E_CLUSTER_NAME:?ATLAS_E2E_CLUSTER_NAME is required by configs/ops/env.schema.json}"
mkdir -p "$(dirname "$log_file")"

if [ "$reuse" = "1" ] && [ -f "$snapshot_json" ]; then
  if ops_context_guard "$profile" >/dev/null 2>&1 \
    && ops_kubectl get ns "$atlas_ns" >/dev/null 2>&1 \
    && ATLAS_HEALTH_REPORT_FORMAT=json ./ops/stack/scripts/health_report.sh "$atlas_ns" "$health_json" >/dev/null 2>&1; then
    duration="$(( $(date +%s) - start_ts ))"
    ops_write_lane_report "stack" "$RUN_ID" "pass" "$duration" "$log_file"
    echo "stack-up reuse hit: healthy snapshot validated for namespace=${atlas_ns}" >"$log_file"
    exit 0
  fi
fi

if ! (
  make -s ops-env-validate
  make -s ops-kind-up
  ./ops/stack/kind/context_guard.sh
  ./ops/stack/kind/namespace_guard.sh
  make -s ops-kind-version-check
  make -s ops-kubectl-version-check
  make -s ops-helm-version-check
  if [ "${ATLAS_KIND_REGISTRY_ENABLE:-0}" = "1" ]; then make -s ops-kind-registry-up; fi
  ./ops/stack/scripts/install.sh
  make -s ops-cluster-sanity
) >"$log_file" 2>&1; then
  status="fail"
fi
ATLAS_HEALTH_REPORT_FORMAT=json ./ops/stack/scripts/health_report.sh "$atlas_ns" "$health_json" >/dev/null || true
duration="$(( $(date +%s) - start_ts ))"
ops_write_lane_report "stack" "$RUN_ID" "$status" "$duration" "$log_file"

if [ "$status" = "pass" ]; then
  mkdir -p "$(dirname "$snapshot_json")"
  python3 - <<PY > "$snapshot_json"
import json, datetime
print(json.dumps({
  "schema_version": 1,
  "captured_at": datetime.datetime.now(datetime.timezone.utc).isoformat(),
  "profile": "${profile}",
  "cluster": "${atlas_cluster}",
  "namespace": "${atlas_ns}",
  "health_report": "${health_json}",
  "run_id": "${RUN_ID}",
  "healthy": True
}, indent=2, sort_keys=True))
PY
fi

[ "$status" = "pass" ] || exit 1
