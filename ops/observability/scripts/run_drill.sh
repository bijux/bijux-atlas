#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: run a named observability drill from manifest with deterministic warmup/cleanup and result artifact.
# stability: public
# called-by: ops/observability/tests/test_drills.sh, make ops-drill-runner
set -euo pipefail
# shellcheck source=ops/_lib/common.sh
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../_lib" && pwd)/common.sh"

DRILL_NAME="${1:-}"
if [ -z "$DRILL_NAME" ]; then
  echo "usage: $0 <drill-name>" >&2
  exit 2
fi

MANIFEST="${REPO_ROOT}/ops/observability/drills/drills.json"
RESULT_SCHEMA="${REPO_ROOT}/ops/observability/drills/result.schema.json"
OUT_DIR="${REPO_ROOT}/artifacts/observability/drills"
OPS_OBS_DIR="${REPO_ROOT}/artifacts/ops/observability"
mkdir -p "$OUT_DIR" "$OPS_OBS_DIR"

read_manifest_field() {
  local field="$1"
  DRILL_NAME="$DRILL_NAME" FIELD="$field" python3 - <<'PY'
import json,os,sys
name=os.environ['DRILL_NAME']
field=os.environ['FIELD']
d=json.load(open('ops/observability/drills/drills.json'))
match=[x for x in d.get('drills',[]) if x.get('name')==name]
if not match:
    print(f'drill not found: {name}', file=sys.stderr)
    raise SystemExit(3)
v=match[0].get(field)
if isinstance(v,bool):
    print('true' if v else 'false')
elif v is None:
    print('')
elif isinstance(v,list):
    import json as _j
    print(_j.dumps(v))
else:
    print(v)
PY
}

SCRIPT_REL="$(read_manifest_field script)"
TIMEOUT_SECONDS="$(read_manifest_field timeout_seconds)"
WARMUP="$(read_manifest_field warmup)"
CLEANUP="$(read_manifest_field cleanup)"
EXPECTED_SIGNALS="$(read_manifest_field expected_signals)"

if [ -z "$SCRIPT_REL" ]; then
  echo "manifest missing script for $DRILL_NAME" >&2
  exit 4
fi
SCRIPT_PATH="${REPO_ROOT}/${SCRIPT_REL}"

cleanup() {
  # deterministic cleanup to baseline
  "${REPO_ROOT}/ops/stack/faults/block-minio.sh" off >/dev/null 2>&1 || true
  ns="${ATLAS_NS:-${ATLAS_E2E_NAMESPACE:-atlas-e2e}}"
  kubectl -n "$ns" delete pod toxiproxy-latency --ignore-not-found >/dev/null 2>&1 || true
}

if [ "$WARMUP" = "true" ]; then
  curl -fsS "${ATLAS_BASE_URL:-http://127.0.0.1:18080}/healthz" >/dev/null || true
  curl -fsS "${ATLAS_BASE_URL:-http://127.0.0.1:18080}/v1/version" >/dev/null || true
fi

started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
status="pass"
if ! ops_timeout_run "${TIMEOUT_SECONDS:-120}" "$SCRIPT_PATH"; then
  status="fail"
fi

"${REPO_ROOT}/ops/observability/scripts/snapshot_metrics.sh" "$OPS_OBS_DIR"
"${REPO_ROOT}/ops/observability/scripts/snapshot_traces.sh" "$OPS_OBS_DIR"
log_snapshot="${OPS_OBS_DIR}/drill-${DRILL_NAME}.logs.txt"
kubectl -n "${ATLAS_E2E_NAMESPACE:-atlas-e2e}" logs -l app.kubernetes.io/instance="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}" --all-containers --tail=2000 > "$log_snapshot" 2>/dev/null || true
python3 "${REPO_ROOT}/ops/observability/scripts/validate_logs_schema.py" --namespace "${ATLAS_E2E_NAMESPACE:-atlas-e2e}" --release "${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}" --strict-live || status="fail"

if [ "$status" = "pass" ]; then
  EXPECTED_SIGNALS="$EXPECTED_SIGNALS" DRILL_NAME="$DRILL_NAME" python3 - <<'PY' || status="fail"
import json, os, pathlib, re, sys
signals = json.loads(os.environ["EXPECTED_SIGNALS"])
root = pathlib.Path(".")
metrics = (root / "artifacts/ops/observability/metrics.prom").read_text(encoding="utf-8", errors="replace") if (root / "artifacts/ops/observability/metrics.prom").exists() else ""
traces = (root / "artifacts/ops/observability/traces.snapshot.log").read_text(encoding="utf-8", errors="replace").lower() if (root / "artifacts/ops/observability/traces.snapshot.log").exists() else ""
log_paths = sorted((root / "artifacts/ops/observability").glob("drill-*.logs.txt"))
logs = "\n".join(p.read_text(encoding="utf-8", errors="replace").lower() for p in log_paths)
for signal in signals:
    kind, _, value = signal.partition(":")
    if kind == "metric":
      if value not in metrics:
          print(f"missing expected metric signal: {value}", file=sys.stderr); sys.exit(1)
    elif kind == "trace":
      if value.lower() not in traces:
          print(f"missing expected trace signal: {value}", file=sys.stderr); sys.exit(1)
    elif kind == "log":
      if value.lower() not in logs:
          print(f"missing expected log signal: {value}", file=sys.stderr); sys.exit(1)
    elif kind == "validator":
      # validator signals are asserted by drill script behavior itself
      continue
    else:
      print(f"unknown expected signal kind: {signal}", file=sys.stderr); sys.exit(1)
PY
fi

if [ "$CLEANUP" = "true" ]; then
  cleanup
fi

ended_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
result_file="${OUT_DIR}/${DRILL_NAME}.result.json"

DRILL_NAME="$DRILL_NAME" STARTED_AT="$started_at" ENDED_AT="$ended_at" STATUS="$status" EXPECTED_SIGNALS="$EXPECTED_SIGNALS" RESULT_FILE="$result_file" LOG_SNAPSHOT="$log_snapshot" python3 - <<'PY'
import json,os,re
from pathlib import Path
result={
  "schema_version":1,
  "drill":os.environ['DRILL_NAME'],
  "started_at":os.environ['STARTED_AT'],
  "ended_at":os.environ['ENDED_AT'],
  "status":"pass" if os.environ['STATUS']=="pass" else "fail",
  "snapshot_paths":{
    "metrics":"artifacts/ops/observability/metrics.prom",
    "traces":"artifacts/ops/observability/traces.snapshot.log",
    "logs":os.environ["LOG_SNAPSHOT"]
  },
  "trace_ids":[],
  "expected_signals":json.loads(os.environ['EXPECTED_SIGNALS'])
}
trace_path=Path('artifacts/ops/observability/traces.snapshot.log')
if trace_path.exists():
  text=trace_path.read_text(encoding='utf-8',errors='replace')
  ids=sorted(set(re.findall(r'[0-9a-f]{16,32}', text)))
  result['trace_ids']=ids[:20]
Path(os.environ['RESULT_FILE']).write_text(json.dumps(result,indent=2,sort_keys=True)+'\n',encoding='utf-8')
PY

python3 - "$result_file" <<'PY'
import json,sys
from pathlib import Path
schema=json.load(open('ops/observability/drills/result.schema.json'))
result=json.load(open(sys.argv[1]))
required=set(schema['required'])
missing=sorted(required-set(result.keys()))
if missing:
  print('result schema validation failed: missing keys', missing, file=sys.stderr)
  raise SystemExit(1)
print('drill result schema validation passed')
PY

if [ "$status" != "pass" ]; then
  echo "drill failed: $DRILL_NAME" >&2
  exit 1
fi

echo "drill passed: $DRILL_NAME ($result_file)"
