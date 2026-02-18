#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

install_chart --set store.credentialsSecretName=atlas-rot-secret --set store.manageCredentialsSecret=true
wait_ready

pod_before="$(pod_name)"
rv_before="$(kubectl -n "$NS" get secret atlas-rot-secret -o jsonpath='{.metadata.resourceVersion}')"

kubectl -n "$NS" patch secret atlas-rot-secret --type merge -p '{"stringData":{"ATLAS_STORE_ACCESS_KEY_ID":"rotated-access","ATLAS_STORE_SECRET_ACCESS_KEY":"rotated-secret"}}' >/dev/null
rv_after="$(kubectl -n "$NS" get secret atlas-rot-secret -o jsonpath='{.metadata.resourceVersion}')"
[ "$rv_after" != "$rv_before" ] || { echo "secret rotation did not update resourceVersion" >&2; exit 1; }

kubectl -n "$NS" rollout restart deployment/"$SERVICE_NAME" >/dev/null
kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout=180s >/dev/null
pod_after="$(pod_name)"
[ "$pod_before" != "$pod_after" ] || { echo "pod did not roll after secret rotation" >&2; exit 1; }

echo "secrets rotation gate passed"
