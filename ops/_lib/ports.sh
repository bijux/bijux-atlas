#!/usr/bin/env bash
# Purpose: canonical deterministic port and URL helpers.
set -euo pipefail

ops_port_atlas() { printf '%s\n' "${ATLAS_PORT:-18080}"; }
ops_port_prometheus() { printf '%s\n' "${ATLAS_PROM_PORT:-19090}"; }
ops_port_grafana() { printf '%s\n' "${ATLAS_GRAFANA_PORT:-13000}"; }

ops_url_atlas() {
  printf '%s\n' "${ATLAS_BASE_URL:-http://127.0.0.1:$(ops_port_atlas)}"
}

ops_url_grafana() {
  printf '%s\n' "${ATLAS_GRAFANA_URL:-http://127.0.0.1:$(ops_port_grafana)}"
}

ops_url_prometheus() {
  printf '%s\n' "${ATLAS_PROM_URL:-http://127.0.0.1:$(ops_port_prometheus)}"
}

ops_ports_publish_json() {
  local out="${1:-${OPS_RUN_DIR:-artifacts/ops/manual}/ports.json}"
  mkdir -p "$(dirname "$out")"
  cat >"$out" <<EOF
{
  "atlas": {"port": $(ops_port_atlas), "url": "$(ops_url_atlas)"},
  "prometheus": {"port": $(ops_port_prometheus), "url": "$(ops_url_prometheus)"},
  "grafana": {"port": $(ops_port_grafana), "url": "$(ops_url_grafana)"}
}
EOF
}
