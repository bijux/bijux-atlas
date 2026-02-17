#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
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

echo "secrets gate passed"
