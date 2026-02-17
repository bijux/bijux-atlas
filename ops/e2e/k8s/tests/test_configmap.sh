#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm; need kubectl

# Define contract: invalid config values must fail startup quickly (CrashLoopBackOff/Error).
install_chart --set concurrency.cheap=not-a-number
sleep 8
POD="$(pod_name || true)"
[ -n "$POD" ] || { echo "no pod created for configmap test" >&2; exit 1; }
STATE="$(kubectl -n "$NS" get pod "$POD" -o jsonpath='{.status.containerStatuses[0].state.waiting.reason}' || true)"
[ "$STATE" = "CrashLoopBackOff" ] || [ "$STATE" = "Error" ] || {
  echo "expected startup failure for invalid configmap value, got: $STATE" >&2
  exit 1
}

echo "configmap gate passed"
