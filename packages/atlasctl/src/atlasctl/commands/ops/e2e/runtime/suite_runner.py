from __future__ import annotations

import subprocess
import sys

SCRIPT = r'''set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"
LAYER_CONTRACT="${ATLAS_LAYER_CONTRACT_PATH:-$ROOT/ops/_meta/layer-contract.json}"
ops_layer_contract_get() {
  local key="$1"
  python3 - "$key" "$LAYER_CONTRACT" <<'PY'
import json, sys
key = sys.argv[1]
path = sys.argv[2]
obj = json.load(open(path, encoding='utf-8'))
cur = obj
for part in key.split('.'):
    if isinstance(cur, dict) and part in cur:
        cur = cur[part]
    else:
        raise SystemExit(f"missing key: {key}")
print(json.dumps(cur, sort_keys=True) if isinstance(cur, (dict, list)) else cur)
PY
}
ops_layer_ns_e2e() { ops_layer_contract_get "namespaces.e2e"; }
ops_context_guard() {
  local profile="${1:-kind}"
  if [ "${I_KNOW_WHAT_I_AM_DOING:-0}" = "1" ] || [ "${ALLOW_NON_KIND:-0}" = "1" ]; then return 0; fi
  local ctx; ctx="$(kubectl config current-context 2>/dev/null || true)"
  case "$profile" in
    kind) case "$ctx" in kind-*|*kind*) return 0;; esac; echo "invalid kubectl context '$ctx' for profile=kind" >&2; return 2 ;;
    perf) [ -n "$ctx" ] && return 0; echo "missing kubectl context for profile=perf" >&2; return 2 ;;
    *) echo "unknown profile '$profile'" >&2; return 2 ;;
  esac
}
ops_env_load() {
  python3 ./packages/atlasctl/src/atlasctl/layout_checks/validate_ops_env.py --schema "${OPS_ENV_SCHEMA:-configs/ops/env.schema.json}" >/dev/null
  export RUN_ID="${RUN_ID:-${OPS_RUN_ID:-}}"
  export ARTIFACT_DIR="${ARTIFACT_DIR:-${OPS_RUN_DIR:-}}"
  if [ -z "${RUN_ID:-}" ] || [ -z "${ARTIFACT_DIR:-}" ]; then echo "RUN_ID and ARTIFACT_DIR must be set" >&2; return 2; fi
}
ops_version_guard() { python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py "$@"; }
ops_entrypoint_start() { :; }
# lightweight run-id/artifact defaults when shell helper stack is absent
if [ -z "${OPS_RUN_ID:-}" ]; then export OPS_RUN_ID="${RUN_ID:-local}"; fi
if [ -z "${OPS_RUN_DIR:-}" ]; then export OPS_RUN_DIR="$ROOT/artifacts/ops/${OPS_RUN_ID}"; fi
mkdir -p "$OPS_RUN_DIR"
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "e2e-suite-runner"
ops_version_guard python3 kubectl

SUITE="smoke"
SCENARIO=""
FAST_MODE=0
NO_DEPLOY=0
PROFILE_ARG="${PROFILE:-kind}"

while [ $# -gt 0 ]; do
  case "$1" in
    --suite)
      SUITE="${2:?missing suite id}"
      shift 2
      ;;
    --fast)
      FAST_MODE=1
      shift
      ;;
    --scenario)
      SCENARIO="${2:?missing scenario id}"
      shift 2
      ;;
    --no-deploy)
      NO_DEPLOY=1
      shift
      ;;
    --profile)
      PROFILE_ARG="${2:?missing profile id}"
      shift 2
      ;;
    *)
      echo "unknown argument: $1" >&2
      exit 2
      ;;
  esac
done

MANIFEST="$ROOT/ops/e2e/suites/suites.json"
REPORT_DIR="$OPS_RUN_DIR/e2e"
mkdir -p "$REPORT_DIR"
REPORT_JSON="$REPORT_DIR/report.json"
LOG_FILE="$REPORT_DIR/run.log"
: > "$LOG_FILE"

if ! ops_context_guard "$PROFILE_ARG"; then
  if [ "$NO_DEPLOY" = "1" ]; then
    echo "--no-deploy set but context/profile is not ready: $PROFILE_ARG" >&2
    exit 1
  fi
  if [ "$PROFILE_ARG" = "kind" ]; then
    echo "e2e runner: bootstrapping kind stack for profile=$PROFILE_ARG" | tee -a "$LOG_FILE"
    make -s ops-stack-up PROFILE=kind >>"$LOG_FILE" 2>&1
  fi
fi
ops_context_guard "$PROFILE_ARG"

if [ "$NO_DEPLOY" != "1" ]; then
  make -s ops-deploy PROFILE="$PROFILE_ARG" >>"$LOG_FILE" 2>&1
else
  echo "e2e runner: no-deploy mode enabled" | tee -a "$LOG_FILE"
fi

suite_caps_json="$(python3 - "$MANIFEST" "$SUITE" <<'PY'
import json, sys
m = json.load(open(sys.argv[1], encoding='utf-8'))
sid = sys.argv[2]
for suite in m.get('suites', []):
    if suite.get('id') == sid:
        print(json.dumps(suite.get('required_capabilities', [])))
        break
else:
    raise SystemExit(f"unknown e2e suite: {sid}")
PY
)"

check_capability() {
  local cap="$1"
  case "$cap" in
    k8s)
      command -v kubectl >/dev/null 2>&1 || { echo "missing capability: kubectl" >&2; return 1; }
      ;;
    stack)
      kubectl get ns "${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_e2e)}" >/dev/null 2>&1 || {
        echo "missing stack capability: namespace not present" >&2
        return 1
      }
      ;;
    obs)
      if [ "${ATLAS_E2E_ENABLE_OTEL:-0}" != "1" ] && ! kubectl -n "${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_e2e)}" get deploy/otel-collector >/dev/null 2>&1; then
        echo "missing obs capability: set ATLAS_E2E_ENABLE_OTEL=1 or deploy otel-collector" >&2
        return 1
      fi
      ;;
    toxiproxy)
      if [ "${ATLAS_E2E_ENABLE_TOXIPROXY:-0}" != "1" ] && ! kubectl -n "${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_e2e)}" get deploy/toxiproxy >/dev/null 2>&1; then
        echo "missing toxiproxy capability: set ATLAS_E2E_ENABLE_TOXIPROXY=1 or deploy toxiproxy" >&2
        return 1
      fi
      ;;
    redis)
      if [ "${ATLAS_E2E_ENABLE_REDIS:-0}" != "1" ] && ! kubectl -n "${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_e2e)}" get deploy/redis >/dev/null 2>&1; then
        echo "missing redis capability: set ATLAS_E2E_ENABLE_REDIS=1 or deploy redis" >&2
        return 1
      fi
      ;;
    datasets|deploy)
      :
      ;;
    *)
      echo "unknown capability: $cap" >&2
      return 1
      ;;
  esac
}

python3 - "$suite_caps_json" <<'PY' >"$REPORT_DIR/.suite_caps"
import json,sys
for c in json.loads(sys.argv[1]):
    print(c)
PY
while IFS= read -r cap; do
  [ -n "$cap" ] || continue
  check_capability "$cap"
done < "$REPORT_DIR/.suite_caps"

scenarios_json="$(python3 - "$MANIFEST" "$SUITE" <<'PY'
import json, sys
m = json.load(open(sys.argv[1], encoding='utf-8'))
sid = sys.argv[2]
for suite in m.get('suites', []):
    if suite.get('id') == sid:
        print(json.dumps(suite.get('scenarios', [])))
        break
else:
    raise SystemExit(f"unknown e2e suite: {sid}")
PY
)"

if [ -n "$SCENARIO" ]; then
  scenarios_json="$(python3 - "$scenarios_json" "$SCENARIO" <<'PY'
import json,sys
arr = json.loads(sys.argv[1])
sid = sys.argv[2]
for item in arr:
    if item.get("id") == sid:
        print(json.dumps([item]))
        break
else:
    raise SystemExit(f"unknown scenario `{sid}` for selected suite")
PY
)"
fi

STATUS="pass"
SCENARIO_REPORT="$REPORT_DIR/scenarios.jsonl"
: > "$SCENARIO_REPORT"

python3 - "$scenarios_json" <<'PY' >"$REPORT_DIR/.scenario_count"
import json,sys
print(len(json.loads(sys.argv[1])))
PY
count="$(cat "$REPORT_DIR/.scenario_count")"
for idx in $(seq 0 $((count - 1))); do
  scenario_json="$(python3 - "$scenarios_json" "$idx" <<'PY'
import json,sys
arr=json.loads(sys.argv[1])
print(json.dumps(arr[int(sys.argv[2])]))
PY
)"
  sid="$(python3 - "$scenario_json" <<'PY'
import json,sys
print(json.loads(sys.argv[1])['id'])
PY
)"
  destructive="$(python3 - "$scenario_json" <<'PY'
import json,sys
print('1' if json.loads(sys.argv[1]).get('destructive') else '0')
PY
)"
  if [ "$FAST_MODE" = "1" ] && [ "$destructive" = "1" ]; then
    printf '{"scenario":"%s","status":"skipped","reason":"fast-mode","destructive":true}\n' "$sid" >> "$SCENARIO_REPORT"
    continue
  fi

  python3 - "$scenario_json" <<'PY' >"$REPORT_DIR/.scenario_caps"
import json,sys
for c in json.loads(sys.argv[1]).get('capabilities',[]):
    print(c)
PY
  while IFS= read -r cap; do
    [ -n "$cap" ] || continue
    check_capability "$cap"
  done < "$REPORT_DIR/.scenario_caps"

  cmd="$(python3 - "$scenario_json" <<'PY'
import json,sys
print(json.loads(sys.argv[1])['runner'])
PY
)"
  start_ts="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  start_s="$(date +%s)"
  if bash -lc "$cmd" >>"$LOG_FILE" 2>&1; then
    end_s="$(date +%s)"
    dur="$((end_s - start_s))"
    printf '{"scenario":"%s","status":"pass","start":"%s","duration_seconds":%s,"budget":%s}\n' "$sid" "$start_ts" "$dur" "$(python3 - "$scenario_json" <<'PY'
import json,sys
print(json.dumps(json.loads(sys.argv[1]).get('budget',{}), sort_keys=True))
PY
)" >> "$SCENARIO_REPORT"
  else
    STATUS="fail"
    end_s="$(date +%s)"
    dur="$((end_s - start_s))"
    printf '{"scenario":"%s","status":"fail","start":"%s","duration_seconds":%s,"budget":%s}\n' "$sid" "$start_ts" "$dur" "$(python3 - "$scenario_json" <<'PY'
import json,sys
print(json.dumps(json.loads(sys.argv[1]).get('budget',{}), sort_keys=True))
PY
)" >> "$SCENARIO_REPORT"
    break
  fi
done

python3 - "$REPORT_JSON" "$RUN_ID" "$SUITE" "$SCENARIO" "$PROFILE_ARG" "$FAST_MODE" "$NO_DEPLOY" "$STATUS" "$SCENARIO_REPORT" "$LOG_FILE" <<'PY'
import json,sys
out, run_id, suite, scenario, profile, fast_mode, no_deploy, status, scen_path, log_path = sys.argv[1:]
scenarios = []
with open(scen_path, encoding='utf-8') as f:
    for line in f:
        line=line.strip()
        if line:
            scenarios.append(json.loads(line))
obj = {
    "run_id": run_id,
    "suite": suite,
    "scenario": scenario or None,
    "profile": profile,
    "fast_mode": fast_mode == "1",
    "no_deploy": no_deploy == "1",
    "status": status,
    "scenarios": scenarios,
    "artifacts": {
        "scenario_report": scen_path,
        "log": log_path,
    },
}
with open(out, 'w', encoding='utf-8') as f:
    json.dump(obj, f, indent=2, sort_keys=True)
    f.write('\n')
print(out)
PY

if [ "$STATUS" != "pass" ]; then
  exit 1
fi
'''


def main() -> int:
    args = sys.argv[1:]
    return subprocess.call(["bash", "-lc", SCRIPT, "--", *args])


if __name__ == '__main__':
    raise SystemExit(main())
