#!/usr/bin/env bash
set -euo pipefail

ROOT="$(git rev-parse --show-toplevel)"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e-${USER:-local}}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
VALUES="${ATLAS_E2E_VALUES_FILE:-$ROOT/ops/k8s/values/local.yaml}"
CHART="$ROOT/ops/k8s/charts/bijux-atlas"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$RELEASE-bijux-atlas}"
BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"

need() { command -v "$1" >/dev/null 2>&1 || { echo "$1 required" >&2; exit 1; }; }

setup_test_traps() {
  trap '"$ROOT"/ops/_lib/k8s-test-report.sh "$NS" "$RELEASE" || true' ERR
  trap 'stop_port_forward; cleanup_namespace' EXIT
}

cleanup_namespace() {
  if [ "${ATLAS_E2E_KEEP_NAMESPACE:-0}" = "1" ]; then
    return 0
  fi
  kubectl delete ns "$NS" --ignore-not-found >/dev/null 2>&1 || true
}

wait_ready() {
  kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout=300s >/dev/null
}

wait_kubectl_condition() {
  local kind="$1"
  local name="$2"
  local cond="$3"
  local timeout="${4:-120s}"
  if ! kubectl -n "$NS" wait --for="condition=${cond}" --timeout="$timeout" "${kind}/${name}" >/dev/null; then
    echo "kubectl wait failed: ${kind}/${name} condition=${cond} timeout=${timeout}" >&2
    return 1
  fi
}

wait_for_http() {
  local url="$1"
  local expected="${2:-200}"
  local timeout_s="${3:-90}"
  local start
  start="$(date +%s)"
  while true; do
    code="$(curl -s -o /dev/null -w '%{http_code}' "$url" || true)"
    if [ "$code" = "$expected" ]; then
      return 0
    fi
    now="$(date +%s)"
    if [ $((now - start)) -ge "$timeout_s" ]; then
      echo "timeout waiting for $url status=$expected, last=$code" >&2
      return 1
    fi
    sleep 2
  done
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
  kubectl get ns "$NS" >/dev/null 2>&1 || kubectl create ns "$NS" >/dev/null
  helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" --wait --timeout 5m "$@"
}

pod_name() {
  kubectl -n "$NS" get pod -l app.kubernetes.io/instance="$RELEASE" -o jsonpath='{.items[0].metadata.name}'
}
