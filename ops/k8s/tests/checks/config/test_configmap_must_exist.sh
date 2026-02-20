#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

install_chart
wait_ready
cm="${SERVICE_NAME}-config"
kubectl -n "$NS" get configmap "$cm" >/dev/null
for key in ATLAS_CONFIG_RELEASE_ID ATLAS_CONFIG_SCHEMA_VERSION ATLAS_REQUEST_TIMEOUT_MS ATLAS_SQL_TIMEOUT_MS; do
  val="$(kubectl -n "$NS" get configmap "$cm" -o jsonpath="{.data.${key}}" || true)"
  [ -n "$val" ] || { echo "configmap must-exist check failed: missing key $key" >&2; exit 1; }
done

echo "configmap must exist contract passed"
