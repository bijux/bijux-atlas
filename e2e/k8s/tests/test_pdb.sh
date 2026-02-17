#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need kubectl

wait_ready
kubectl -n "$NS" get pdb "$SERVICE_NAME" >/dev/null
MIN_AVAIL="$(kubectl -n "$NS" get pdb "$SERVICE_NAME" -o jsonpath='{.spec.minAvailable}')"
[ -n "$MIN_AVAIL" ] || { echo "pdb minAvailable missing" >&2; exit 1; }

echo "pdb gate passed"
