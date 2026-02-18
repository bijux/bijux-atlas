#!/usr/bin/env bash
# Purpose: canonical local URL helpers for atlas and observability endpoints.
# Inputs: optional ATLAS_*_URL env vars.
# Outputs: resolved URL values.
set -euo pipefail

ops_url_atlas() {
  printf '%s\n' "${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
}

ops_url_grafana() {
  printf '%s\n' "${ATLAS_GRAFANA_URL:-http://127.0.0.1:3000}"
}

ops_url_prometheus() {
  printf '%s\n' "${ATLAS_PROM_URL:-http://127.0.0.1:19090}"
}
