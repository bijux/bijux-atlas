#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need kubectl

wait_ready
kubectl -n "$NS" get hpa "$SERVICE_NAME" >/dev/null
kubectl -n "$NS" get hpa "$SERVICE_NAME" -o jsonpath='{.spec.maxReplicas}' | grep -Eq '^[0-9]+$'

echo "hpa gate passed"
