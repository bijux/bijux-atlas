#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$(dirname "$0")/common.sh"
setup_test_traps
need curl

OFFLINE="$ROOT/ops/k8s/values/offline.yaml"
install_chart -f "$OFFLINE"
wait_ready
with_port_forward 18080

# Offline mode must remain healthy even with unreachable remote store endpoint.
tmp_bad_store="$(mktemp)"
cat >"$tmp_bad_store" <<YAML
store:
  endpoint: http://unreachable-store.invalid:9000
catalog:
  cacheOnlyMode: true
YAML
helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" -f "$OFFLINE" -f "$tmp_bad_store" >/dev/null
wait_ready
curl -fsS "$BASE_URL/readyz" >/dev/null
rm -f "$tmp_bad_store"

echo "offline profile gate passed"
