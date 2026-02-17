#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
VALUES="${ATLAS_E2E_VALUES_FILE:-$ROOT/ops/k8s/values/local.yaml}"
CHART="$ROOT/charts/bijux-atlas"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$RELEASE-bijux-atlas}"
BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"

need() { command -v "$1" >/dev/null 2>&1 || { echo "$1 required" >&2; exit 1; }; }

wait_ready() {
  kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout=300s
}

with_port_forward() {
  PF_LOCAL_PORT="${1:-18080}"
  kubectl -n "$NS" port-forward "svc/$SERVICE_NAME" "${PF_LOCAL_PORT}:8080" >/tmp/bijux-atlas-port-forward.log 2>&1 &
  PF_PID=$!
  sleep 2
}

stop_port_forward() {
  if [ -n "${PF_PID:-}" ]; then
    kill "$PF_PID" >/dev/null 2>&1 || true
    wait "$PF_PID" 2>/dev/null || true
    unset PF_PID
  fi
}

install_chart() {
  helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" --wait --timeout 5m "$@"
}

pod_name() {
  kubectl -n "$NS" get pod -l app.kubernetes.io/instance="$RELEASE" -o jsonpath='{.items[0].metadata.name}'
}
