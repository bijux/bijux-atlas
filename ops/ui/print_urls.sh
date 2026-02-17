#!/usr/bin/env bash
# Purpose: print local ops service URLs for manual inspection.
# Inputs: optional ATLAS_BASE_URL, GRAFANA_PORT.
# Outputs: stdout URLs only.
set -euo pipefail

base_url="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
grafana_url="${ATLAS_GRAFANA_URL:-http://127.0.0.1:3000}"
prom_url="${ATLAS_PROM_URL:-http://127.0.0.1:9090}"

printf 'atlas=%s\n' "$base_url"
printf 'grafana=%s\n' "$grafana_url"
printf 'prometheus=%s\n' "$prom_url"
