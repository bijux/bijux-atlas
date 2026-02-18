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

probe_http() {
  local url="$1"
  local label="$2"
  for _ in $(seq 1 30); do
    if curl -sS -o /dev/null "$url" >/dev/null 2>&1; then
      echo "$label reachable: $url"
      return 0
    fi
    sleep 1
  done
  echo "$label unreachable: $url" >&2
  return 1
}

case "$PROFILE" in
  local-compose)
    prom_port="$(python3 -c 'import json;print(json.load(open("configs/ops/observability-pack.json"))["ports"]["prometheus"])')"
    grafana_port="$(python3 -c 'import json;print(json.load(open("configs/ops/observability-pack.json"))["ports"]["grafana"])')"
    otel_http_port="$(python3 -c 'import json;print(json.load(open("configs/ops/observability-pack.json"))["ports"]["otel_http"])')"
    probe_http "http://127.0.0.1:${prom_port}/-/ready" "prometheus"
    probe_http "http://127.0.0.1:${grafana_port}/api/health" "grafana"
    probe_http "http://127.0.0.1:${otel_http_port}/" "otel-collector"
    ;;
  kind | cluster)
    ns="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
    ops_kubectl_wait_condition "$ns" deploy prometheus available "${OPS_WAIT_TIMEOUT:-180s}"
    ops_kubectl_wait_condition "$ns" deploy grafana available "${OPS_WAIT_TIMEOUT:-180s}"
    ops_kubectl_wait_condition "$ns" deploy otel-collector available "${OPS_WAIT_TIMEOUT:-180s}"
    ;;
  *)
    echo "unknown profile: $PROFILE (expected: local-compose|kind|cluster)" >&2
    exit 1
    ;;
esac

echo "observability pack verified (profile=$PROFILE)"
