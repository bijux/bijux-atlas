#!/usr/bin/env bash
# Purpose: print local ops service URLs for manual inspection.
# Inputs: optional ATLAS_BASE_URL, GRAFANA_PORT.
# Outputs: stdout URLs only.
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
# shellcheck source=ops/_lib/ports.sh
source "$ROOT/ops/_lib/ports.sh"

printf 'atlas=%s\n' "$(ops_url_atlas)"
printf 'grafana=%s\n' "$(ops_url_grafana)"
printf 'prometheus=%s\n' "$(ops_url_prometheus)"
