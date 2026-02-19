#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm; need curl

TMP1="$(mktemp)"
cat > "$TMP1" <<YAML
server:
  readinessRequiresCatalog: true
  cachedOnlyMode: false
catalog:
  endpoint: "http://non-existent-catalog.invalid/catalog.json"
YAML
install_chart -f "$TMP1"
sleep 10
with_port_forward 18080
code1="$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/readyz" || true)"
[ "$code1" = "503" ] || { echo "expected 503 without catalog, got $code1" >&2; exit 1; }

TMP2="$(mktemp)"
cat > "$TMP2" <<YAML
server:
  readinessRequiresCatalog: true
  cachedOnlyMode: true
catalog:
  endpoint: "http://non-existent-catalog.invalid/catalog.json"
YAML
install_chart -f "$TMP2"
wait_ready
code2="$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/readyz" || true)"
[ "$code2" = "200" ] || { echo "expected 200 in cached-only mode, got $code2" >&2; exit 1; }

echo "readiness semantics gate passed"
