#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/_lib/common.sh
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../_lib" && pwd)/common.sh"

PROFILE="${ATLAS_OBS_PROFILE:-kind}"
if [ "${1:-}" = "--profile" ]; then
  PROFILE="${2:-}"
fi
OBS_NS="${ATLAS_OBS_NAMESPACE:-atlas-observability}"

compose_cmd() {
  if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    echo "docker compose"
    return 0
  fi
  if command -v docker-compose >/dev/null 2>&1; then
    echo "docker-compose"
    return 0
  fi
  return 1
}

case "$PROFILE" in
  local-compose)
    cmd="$(compose_cmd)" || {
      echo "local-compose profile requires docker compose or docker-compose" >&2
      exit 1
    }
    $cmd -f "${REPO_ROOT}/ops/obs/pack/compose/docker-compose.yml" down -v
    ;;
  kind | cluster)
    ops_kubectl -n "$OBS_NS" delete -f "${REPO_ROOT}/ops/obs/alerts/slo-burn-rules.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl -n "$OBS_NS" delete -f "${REPO_ROOT}/ops/obs/alerts/atlas-alert-rules.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/otel.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl -n "$OBS_NS" delete configmap atlas-observability-otel-config --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/grafana.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/grafana-config.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/prometheus.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/prometheus-config.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/prometheus-pvc.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/grafana-pvc.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/rbac.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/obs/pack/k8s/namespace.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ;;
  *)
    echo "unknown profile: $PROFILE (expected: local-compose|kind|cluster)" >&2
    exit 1
    ;;
esac

echo "observability pack uninstalled (profile=$PROFILE)"
