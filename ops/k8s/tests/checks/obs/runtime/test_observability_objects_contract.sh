#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need helm grep

obs_on="$(mktemp)"
obs_off="$(mktemp)"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set serviceMonitor.enabled=true --set alertRules.enabled=true >"$obs_on"
grep -q "kind: ServiceMonitor" "$obs_on"
grep -q "kind: PrometheusRule" "$obs_on"

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set serviceMonitor.enabled=false --set alertRules.enabled=false >"$obs_off"
if grep -q "kind: PrometheusRule" "$obs_off"; then
  echo "observability objects contract failed: PrometheusRule rendered when obs disabled" >&2
  exit 1
fi

if helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set serviceMonitor.enabled=false --set alertRules.enabled=true >/tmp/obs-objects-invalid.yaml 2>/tmp/obs-objects-invalid.err; then
  echo "expected render failure when alert rules enabled but service monitor disabled" >&2
  exit 1
fi
grep -q "alertRules.enabled=true requires serviceMonitor.enabled=true" /tmp/obs-objects-invalid.err

echo "observability objects contract passed"
