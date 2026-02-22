#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"

require_bin curl
require_bin kubectl
require_bin python3

OUT_DIR="$ROOT/artifacts/observability"
OPS_OBS_DIR="$ROOT/artifacts/ops/obs"
mkdir -p "$OUT_DIR" "$OPS_OBS_DIR"

# 1) install pack
"$ROOT/ops/obs/scripts/install_pack.sh"
"$ROOT/ops/obs/scripts/verify_pack.sh"
"$ROOT/ops/obs/scripts/pack_health.sh"

# 2) short mixed workload
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
for _ in $(seq 1 5); do
  curl -fsS "$ATLAS_BASE_URL/healthz" >/dev/null || true
  curl -fsS "$ATLAS_BASE_URL/v1/version" >/dev/null || true
  curl -fsS "$ATLAS_BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=3" >/dev/null || true
  curl -fsS "$ATLAS_BASE_URL/v1/transcripts/ENST00000357654?release=110&species=homo_sapiens&assembly=GRCh38" >/dev/null || true
  sleep 1
done

# 3) snapshot metrics + traces + logs
"$ROOT/ops/obs/scripts/snapshot_metrics.sh" "$OPS_OBS_DIR"
"$ROOT/ops/obs/scripts/snapshot_traces.sh" "$OPS_OBS_DIR"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/validate_logs_schema.py" --namespace "${ATLAS_E2E_NAMESPACE:-atlas-e2e}" --release "${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"

# 4) run contracts checks
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_metrics_contract.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_metrics_coverage.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_metrics_drift.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_metrics_golden.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_tracing_contract.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_trace_golden.py"
if [ "${ATLAS_E2E_ENABLE_OTEL:-0}" = "1" ]; then
  python3 "$ROOT/ops/obs/scripts/areas/contracts/check_trace_coverage.py"
fi
python3 "$ROOT/ops/obs/scripts/areas/contracts/extract_trace_exemplars.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/obs/contracts/check_dashboard_contract.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/obs/contracts/check_alerts_contract.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/obs/contracts/lint_runbooks.py"

# minimum coverage threshold (tier-0)
python3 - <<'PY'
import json,re,sys
from pathlib import Path
root=Path('.')
contract=json.loads((root/'ops/obs/contract/metrics-contract.json').read_text())
tier0=[n for n,s in contract.get('required_metric_specs',{}).items() if s.get('criticality')=='tier-0']
text=(root/'artifacts/ops/obs/metrics.prom').read_text(encoding='utf-8',errors='replace')
observed=set(re.findall(r'^((?:bijux|atlas)_[a-zA-Z0-9_]+)\{',text,flags=re.M))
if not tier0:
    print('no tier-0 metrics declared',file=sys.stderr);sys.exit(1)
covered=[m for m in tier0 if m in observed]
ratio=len(covered)/len(tier0)
print(f'tier-0 coverage: {len(covered)}/{len(tier0)} ({ratio:.1%})')
if ratio < 0.95:
    print('tier-0 coverage below threshold 95%',file=sys.stderr)
    print('missing:',', '.join(sorted(set(tier0)-set(covered))),file=sys.stderr)
    sys.exit(1)
PY

# artifacts contract + version stamp
python3 - <<'PY'
import json,subprocess
from pathlib import Path
root=Path('.')
out=root/'artifacts/observability'
out.mkdir(parents=True,exist_ok=True)
ops=root/'artifacts/ops/obs'
for name in ('metrics.prom','traces.snapshot.log','traces.exemplars.log','metrics-drift.md','trace-exemplars.by-scenario.json'):
    src=ops/name
    if src.exists():
        (out/name).write_bytes(src.read_bytes())
required=['metrics.prom','traces.snapshot.log','traces.exemplars.log']
for r in required:
    if not (out/r).exists():
        raise SystemExit(f'missing required artifact: artifacts/observability/{r}')
sha=subprocess.check_output(['git','rev-parse','HEAD'],text=True).strip()
metrics_schema=json.loads((root/'ops/obs/contract/metrics-contract.json').read_text()).get('schema_version')
trace_schema=json.loads((root/'docs/contracts/TRACE_SPANS.json').read_text()).get('schema_version')
logs_schema=json.loads((root/'ops/obs/contract/logs-fields-contract.json').read_text()).get('schema_version')
payload={
 'git_sha':sha,
 'metrics_contract_schema_version':metrics_schema,
 'trace_contract_schema_version':trace_schema,
 'logs_contract_schema_version':logs_schema,
}
(out/'pack-version-stamp.json').write_text(json.dumps(payload,indent=2,sort_keys=True)+'\n',encoding='utf-8')
print('wrote artifacts/observability/pack-version-stamp.json')
PY
"$ROOT/ops/obs/scripts/export_pack_bundle.sh"
"$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/write_pack_conformance_report.py"
test -s "$ROOT/artifacts/observability/pack-conformance-report.json"

echo "observability coverage test passed"
