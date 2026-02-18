#!/usr/bin/env sh
set -eu

NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
ENABLE_OTEL="${ATLAS_E2E_ENABLE_OTEL:-0}"

if [ "$ENABLE_OTEL" != "1" ]; then
  echo "otel disabled; skipping trace verification"
  exit 0
fi

POD="$(kubectl -n "$NS" get pod -l app=otel-collector -o jsonpath='{.items[0].metadata.name}')"
# Best-effort signal: collector logs should show request-path spans emitted.
kubectl -n "$NS" logs "$POD" --tail=800 | grep -E "admission_control|dataset_resolve|cache_lookup|store_fetch|open_db|sqlite_query|serialize_response" >/dev/null

echo "trace verification passed"
