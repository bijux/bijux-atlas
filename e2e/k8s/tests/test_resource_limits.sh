#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need kubectl

wait_ready
POD="$(pod_name)"
CPU_LIMIT="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.spec.containers[0].resources.limits.cpu}')"
MEM_LIMIT="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.spec.containers[0].resources.limits.memory}')"
[ -n "$CPU_LIMIT" ] && [ -n "$MEM_LIMIT" ] || { echo "missing resource limits" >&2; exit 1; }

echo "resource limits gate passed"
