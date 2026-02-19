#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl; need helm; need curl

TMP_BAD="$(mktemp)"
TMP_GOOD="$(mktemp)"
trap 'rm -f "$TMP_BAD" "$TMP_GOOD"' EXIT

cat >"$TMP_BAD" <<YAML
catalog:
  readinessRequiresCatalog: true
  cacheOnlyMode: false
  endpoint: "http://non-existent-catalog.invalid/catalog.json"
YAML

cat >"$TMP_GOOD" <<YAML
catalog:
  readinessRequiresCatalog: true
  cacheOnlyMode: true
  endpoint: "http://non-existent-catalog.invalid/catalog.json"
YAML

install_chart -f "$TMP_GOOD"
wait_ready
with_port_forward 18080
curl -fsS "$BASE_URL/readyz" >/dev/null

helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" -f "$TMP_BAD" >/dev/null
sleep 10
ready_bad="$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/readyz" || true)"
[ "$ready_bad" != "200" ] || {
  echo "readiness stayed healthy during bad catalog refresh" >&2
  exit 1
}

helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" -f "$TMP_GOOD" >/dev/null
wait_ready
curl -fsS "$BASE_URL/readyz" >/dev/null

echo "readiness catalog refresh gate passed"
