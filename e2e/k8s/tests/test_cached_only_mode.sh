#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
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
curl -fsS "$BASE_URL/readyz" >/dev/null

echo "cached-only mode gate passed"
