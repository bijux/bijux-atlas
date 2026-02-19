#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm; need kubectl

TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
store:
  credentialsSecretName: atlas-missing-secret
  manageCredentialsSecret: false
YAML

helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" -f "$TMP_VALUES" >/dev/null
sleep 5
POD="$(pod_name || true)"
[ -n "$POD" ] || { echo "no pod created" >&2; exit 1; }
REASON="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.status.containerStatuses[0].state.waiting.reason}' || true)"
[ "$REASON" = "CreateContainerConfigError" ] || { echo "expected CreateContainerConfigError, got: $REASON" >&2; exit 1; }

kubectl -n "$NS" create secret generic atlas-wrong-secret --from-literal=ATLAS_STORE_ACCESS_KEY_ID=bad >/dev/null 2>&1 || true
helm upgrade --install "$RELEASE" "$CHART" -n "$NS" --create-namespace -f "$VALUES" \
  --set store.credentialsSecretName=atlas-wrong-secret \
  --set store.manageCredentialsSecret=false >/dev/null
sleep 10
POD2="$(pod_name || true)"
[ -n "$POD2" ] || { echo "no pod created with wrong secret case" >&2; exit 1; }
STATE2="$(kubectl -n "$NS" get pod "$POD2" -o jsonpath='{.status.containerStatuses[0].state.waiting.reason}' || true)"
[ "$STATE2" = "CreateContainerConfigError" ] || [ "$STATE2" = "CrashLoopBackOff" ] || [ "$STATE2" = "Error" ] || {
  echo "expected startup failure with wrong secret, got: $STATE2" >&2
  exit 1
}

echo "secrets gate passed"
