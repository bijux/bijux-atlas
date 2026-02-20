#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need kubectl

if ! kubectl api-resources | grep -q "^prometheusrules"; then
  echo "PrometheusRule CRD not present; skipping"
  exit 0
fi

install_chart --set serviceMonitor.enabled=true --set alertRules.enabled=true
wait_ready
kubectl -n "$NS" get prometheusrule "$SERVICE_NAME-alerts" >/dev/null

echo "prometheus rule gate passed"
