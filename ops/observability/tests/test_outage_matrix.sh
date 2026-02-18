#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/observability/tests/observability-test-lib.sh"

require_bin curl
require_bin python3

OUT_DIR="$ROOT/artifacts/observability"
OPS_OBS_DIR="$ROOT/artifacts/ops/observability"
mkdir -p "$OUT_DIR" "$OPS_OBS_DIR"

run_and_assert() {
  local scenario="$1"
  local script_path="$2"
  echo "running outage scenario: $scenario"
  "$script_path"
  "$ROOT/ops/observability/scripts/snapshot_metrics.sh" "$OPS_OBS_DIR"
  "$ROOT/ops/observability/scripts/snapshot_traces.sh" "$OPS_OBS_DIR"
  python3 "$ROOT/ops/observability/scripts/validate_logs_schema.py" --namespace "${ATLAS_E2E_NAMESPACE:-atlas-e2e}" --release "${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}" --strict-live
  python3 - <<PY
import re,sys
from pathlib import Path
scenario='$scenario'
metrics=(Path('artifacts/ops/observability/metrics.prom').read_text(encoding='utf-8',errors='replace') if Path('artifacts/ops/observability/metrics.prom').exists() else '')
traces=(Path('artifacts/ops/observability/traces.snapshot.log').read_text(encoding='utf-8',errors='replace').lower() if Path('artifacts/ops/observability/traces.snapshot.log').exists() else '')
# metric expectation per scenario
metric_ok={
 'prom_outage': ('bijux_http_requests_total' in metrics),
 'otel_outage': ('bijux_http_requests_total' in metrics),
 'store_outage': ('bijux_store_breaker_open' in metrics) or ('bijux_store_download_failure_total' in metrics),
 'overload_shedding': ('atlas_shed_total' in metrics) or ('bijux_overload_shedding_active' in metrics),
}[scenario]
if not metric_ok:
    print(f'{scenario}: missing required metric evidence',file=sys.stderr);sys.exit(1)
# trace error span if applicable (otel/store/overload paths)
if scenario in ('otel_outage','store_outage','overload_shedding') and not any(tok in traces for tok in ('error','status=error','store_fetch','admission_control')):
    print(f'{scenario}: missing trace error/span evidence',file=sys.stderr);sys.exit(1)
print(f'{scenario}: metric/trace evidence ok')
PY
}

run_and_assert prom_outage "$ROOT/ops/observability/scripts/prom-outage.sh"
run_and_assert otel_outage "$ROOT/ops/observability/scripts/otel-outage.sh"
run_and_assert store_outage "$ROOT/ops/observability/scripts/store-outage.sh"
run_and_assert overload_shedding "$ROOT/ops/observability/scripts/overload-shedding.sh"

# copy latest evidence into observability artifact contract path
cp -f "$OPS_OBS_DIR"/metrics.prom "$OUT_DIR"/metrics.prom 2>/dev/null || true
cp -f "$OPS_OBS_DIR"/traces.snapshot.log "$OUT_DIR"/traces.snapshot.log 2>/dev/null || true
cp -f "$OPS_OBS_DIR"/traces.exemplars.log "$OUT_DIR"/traces.exemplars.log 2>/dev/null || true
test -s "$OUT_DIR/metrics.prom"
test -s "$OUT_DIR/traces.snapshot.log"
test -s "$OUT_DIR/traces.exemplars.log"

echo "observability outage matrix passed"
