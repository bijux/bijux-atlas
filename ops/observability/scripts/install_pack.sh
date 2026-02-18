#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/_lib/common.sh
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../_lib" && pwd)/common.sh"
# shellcheck source=ops/_lib/ports.sh
source "${REPO_ROOT}/ops/_lib/ports.sh"

PROFILE="${ATLAS_OBS_PROFILE:-kind}"
if [ "${1:-}" = "--profile" ]; then
  PROFILE="${2:-}"
fi

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
    $cmd -f "${REPO_ROOT}/ops/observability/pack/compose/docker-compose.yml" up -d
    ;;
  kind | cluster)
    ops_kubectl apply -f "${REPO_ROOT}/ops/stack/prometheus/prometheus.yaml"
    ops_kubectl apply -f "${REPO_ROOT}/ops/stack/grafana/grafana.yaml"
    ops_kubectl apply -f "${REPO_ROOT}/ops/stack/otel/otel-collector.yaml"
    if ops_kubectl api-resources | grep -q "^prometheusrules"; then
      ops_kubectl apply -f "${REPO_ROOT}/ops/observability/alerts/atlas-alert-rules.yaml"
    else
      echo "PrometheusRule CRD not present; continuing without rule install"
    fi
    if [ "$PROFILE" = "cluster" ] && ! ops_kubectl api-resources | grep -q "^servicemonitors"; then
      echo "cluster profile requested but ServiceMonitor CRD missing" >&2
      exit 1
    fi
    ;;
  *)
    echo "unknown profile: $PROFILE (expected: local-compose|kind|cluster)" >&2
    exit 1
    ;;
esac

echo "observability pack installed (profile=$PROFILE)"
