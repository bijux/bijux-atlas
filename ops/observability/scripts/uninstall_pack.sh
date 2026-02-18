#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/_lib/common.sh
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../_lib" && pwd)/common.sh"

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
    $cmd -f "${REPO_ROOT}/ops/observability/pack/compose/docker-compose.yml" down -v
    ;;
  kind | cluster)
    ops_kubectl delete -f "${REPO_ROOT}/ops/stack/prometheus/prometheus.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/stack/grafana/grafana.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/stack/otel/otel-collector.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ops_kubectl delete -f "${REPO_ROOT}/ops/observability/alerts/atlas-alert-rules.yaml" --ignore-not-found >/dev/null 2>&1 || true
    ;;
  *)
    echo "unknown profile: $PROFILE (expected: local-compose|kind|cluster)" >&2
    exit 1
    ;;
esac

echo "observability pack uninstalled (profile=$PROFILE)"
