#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
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
wait_for_http "$BASE_URL/healthz" 200 60

echo "cached-only mode gate passed"
