#!/usr/bin/env bash
set -euo pipefail

# shellcheck source=ops/_lib/common.sh
source "$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)/common.sh"

ROOT="$REPO_ROOT"
NS="${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_k8s)}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-$(ops_layer_contract_get release_metadata.defaults.release_name)}"
VALUES="${ATLAS_E2E_VALUES_FILE:-$ROOT/ops/k8s/values/local.yaml}"
CHART="$ROOT/ops/k8s/charts/bijux-atlas"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$(ops_layer_service_atlas)}"
CLUSTER_NAME="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
USE_LOCAL_IMAGE="${ATLAS_E2E_USE_LOCAL_IMAGE:-1}"
LOCAL_IMAGE_REF="${ATLAS_E2E_LOCAL_IMAGE:-bijux-atlas:local}"
BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:$(ops_layer_port_atlas)}"

need() { ops_need_cmd "$1"; }

setup_test_traps() {
  trap '"$ROOT"/ops/_lib/k8s-test-report.sh "$NS" "$RELEASE" || true' ERR
  trap 'stop_port_forward; cleanup_namespace' EXIT
}

cleanup_namespace() {
  if [ "${ATLAS_E2E_KEEP_NAMESPACE:-0}" = "1" ]; then
    return 0
  fi
  kubectl delete ns "$NS" --ignore-not-found --wait=false >/dev/null 2>&1 || true
}

wait_ready() {
  kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout=300s >/dev/null
}

wait_kubectl_condition() {
  local kind="$1"
  local name="$2"
  local cond="$3"
  local timeout="${4:-120s}"
  if ! ops_kubectl_wait_condition "$NS" "$kind" "$name" "$cond" "$timeout"; then
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
  PF_LOCAL_PORT="${1:-$(ops_layer_port_atlas)}"
  BASE_URL="http://127.0.0.1:${PF_LOCAL_PORT}"
  kubectl -n "$NS" port-forward "svc/$SERVICE_NAME" "${PF_LOCAL_PORT}:$(ops_layer_port_atlas)" >/tmp/bijux-atlas-port-forward.log 2>&1 &
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
  if ! ops_wait_namespace_termination "$NS" 120; then
    echo "namespace $NS is still terminating after timeout" >&2
    return 1
  fi
  kubectl get ns "$NS" >/dev/null 2>&1 || kubectl create ns "$NS" >/dev/null
  local node_port
  node_port="$(awk '/^[[:space:]]*nodePort:/ {print $2; exit}' "$VALUES" 2>/dev/null || true)"
  if [ -n "$node_port" ] && [ "$node_port" != "null" ]; then
    while read -r ns svc ports; do
      [ -z "$ns" ] && continue
      [ "$ns" = "$NS" ] && continue
      if echo "$ports" | tr ',' '\n' | grep -qx "$node_port"; then
        if [ "$svc" = "${RELEASE}-bijux-atlas" ] && [[ "$ns" == atlas-ops-* ]]; then
          echo "removing stale NodePort owner: ${ns}/${svc} (port ${node_port})"
          helm -n "$ns" uninstall "$RELEASE" >/dev/null 2>&1 || true
          kubectl -n "$ns" delete svc "$svc" --ignore-not-found >/dev/null 2>&1 || true
        else
          echo "nodePort ${node_port} already allocated by ${ns}/${svc}; refusing destructive cleanup" >&2
          return 1
        fi
      fi
    done < <(kubectl get svc -A -o custom-columns=NS:.metadata.namespace,NAME:.metadata.name,NODEPORTS:.spec.ports[*].nodePort --no-headers)
  fi
  local helm_args=(
    upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" --atomic --wait --timeout 5m
  )
  if [ "$USE_LOCAL_IMAGE" = "1" ]; then
    if ! docker image inspect "$LOCAL_IMAGE_REF" >/dev/null 2>&1; then
      docker build -t "$LOCAL_IMAGE_REF" -f "$ROOT/docker/Dockerfile" "$ROOT"
    fi
    kind load docker-image "$LOCAL_IMAGE_REF" --name "$CLUSTER_NAME"
    helm_args+=(
      --set image.repository="${LOCAL_IMAGE_REF%:*}"
      --set image.tag="${LOCAL_IMAGE_REF#*:}"
      --set image.pullPolicy=IfNotPresent
    )
  fi
  if [ "$#" -gt 0 ]; then
    helm_args+=("$@")
  fi

  local attempt=1
  local out_file
  out_file="$(mktemp)"
  while [ "$attempt" -le 2 ]; do
    if helm "${helm_args[@]}" >"$out_file" 2>&1; then
      cat "$out_file"
      rm -f "$out_file"
      return 0
    fi
    if [ "$attempt" -eq 1 ] && grep -Eq 'services ".*" not found: uninstall: Release not loaded: .* release: not found' "$out_file"; then
      cat "$out_file" >&2
      echo "helm atomic rollback race detected; retrying install once after cleanup" >&2
      helm -n "$NS" uninstall "$RELEASE" >/dev/null 2>&1 || true
      kubectl -n "$NS" delete svc "$SERVICE_NAME" --ignore-not-found >/dev/null 2>&1 || true
      attempt=$((attempt + 1))
      sleep 2
      continue
    fi
    cat "$out_file" >&2
    rm -f "$out_file"
    return 1
  done
  cat "$out_file" >&2
  rm -f "$out_file"
  return 1
}

pod_name() {
  kubectl -n "$NS" get pod -l app.kubernetes.io/instance="$RELEASE" -o jsonpath='{.items[0].metadata.name}'
}
