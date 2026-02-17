#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm; need curl

TMP1="$(mktemp)"
cat > "$TMP1" <<YAML
server:
  readinessRequiresCatalog: true
  cachedOnlyMode: false
catalog:
  endpoint: "http://non-existent-catalog.invalid/catalog.json"
YAML
helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" -f "$TMP1" >/dev/null
sleep 10
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
code2="$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/readyz" || true)"
[ "$code2" = "200" ] || { echo "expected 200 in cached-only mode, got $code2" >&2; exit 1; }

echo "readiness semantics gate passed"
