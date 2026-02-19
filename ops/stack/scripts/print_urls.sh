#!/usr/bin/env bash
# Purpose: print local ops service URLs for manual inspection.
# Inputs: optional ATLAS_BASE_URL, GRAFANA_PORT.
# Outputs: stdout URLs only.
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
# shellcheck source=ops/_lib/ports.sh
source "$ROOT/_lib/ports.sh"

ops_ports_publish_json "${OPS_RUN_DIR:-artifacts/ops/manual}/ports.json"

printf 'atlas=%s\n' "$(ops_url_atlas)"
printf 'grafana=%s\n' "$(ops_url_grafana)"
printf 'prometheus=%s\n' "$(ops_url_prometheus)"
