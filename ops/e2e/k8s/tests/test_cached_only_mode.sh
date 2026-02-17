#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need helm; need curl

TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
server:
  cachedOnlyMode: true
catalog:
  endpoint: "http://non-existent-catalog.invalid/catalog.json"
YAML
install_chart -f "$TMP_VALUES"
wait_ready
with_port_forward 18080
wait_for_http "$BASE_URL/readyz" 200 60

echo "cached-only mode gate passed"
