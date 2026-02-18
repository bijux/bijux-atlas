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
    eval "$("${REPO_ROOT}/ops/obs/scripts/pack_ports.sh")"
    prom_port="${ATLAS_PROM_URL##*:}"
    grafana_port="${ATLAS_GRAFANA_URL##*:}"
    otel_http_port="${ATLAS_OTEL_HTTP_URL##*:}"
    probe_http "http://127.0.0.1:${prom_port}/-/ready" "prometheus"
    probe_http "http://127.0.0.1:${grafana_port}/api/health" "grafana"
    probe_http "http://127.0.0.1:${otel_http_port}/" "otel-collector"
    ;;
  kind | cluster)
    ns="${ATLAS_OBS_NAMESPACE:-atlas-observability}"
    ops_kubectl_wait_condition "$ns" deploy atlas-observability-prometheus available "${OPS_WAIT_TIMEOUT:-180s}"
    ops_kubectl_wait_condition "$ns" deploy atlas-observability-grafana available "${OPS_WAIT_TIMEOUT:-180s}"
    ops_kubectl_wait_condition "$ns" deploy atlas-observability-otel available "${OPS_WAIT_TIMEOUT:-180s}"
    pid_file="$(mktemp)"
    ( ops_kubectl -n "$ns" port-forward svc/atlas-observability-prometheus 19090:9090 >/dev/null 2>&1 & echo $! > "$pid_file" )
    pf_pid="$(cat "$pid_file")"; rm -f "$pid_file"
    trap 'kill "$pf_pid" >/dev/null 2>&1 || true' EXIT INT TERM
    for _ in $(seq 1 30); do
      if curl -fsS "http://127.0.0.1:19090/api/v1/targets" | python3 -c 'import json,sys; d=json.load(sys.stdin); a=d.get("data",{}).get("activeTargets",[]); sys.exit(0 if len(a)>0 else 1)'; then
        echo "prometheus targets up"
        break
      fi
      sleep 1
    done
    ;;
  *)
    echo "unknown profile: $PROFILE (expected: local-compose|kind|cluster)" >&2
    exit 1
    ;;
esac

echo "observability pack verified (profile=$PROFILE)"
