#!/usr/bin/env sh
set -eu
. "$(dirname "$0")/common.sh"
need helm; need kubectl

install_chart
wait_ready
REV1="$(helm -n "$NS" history "$RELEASE" -o json | grep -o '"revision":[0-9]*' | tail -n1 | cut -d: -f2)"
helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set server.responseMaxBytes=262144 >/dev/null
REV2="$(helm -n "$NS" history "$RELEASE" -o json | grep -o '"revision":[0-9]*' | tail -n1 | cut -d: -f2)"
[ "$REV2" -gt "$REV1" ]
if helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set image.tag=does-not-exist --wait --timeout 90s >/dev/null 2>&1; then
  echo "expected faulty upgrade to fail" >&2
  exit 1
fi
helm rollback "$RELEASE" "$REV1" -n "$NS" --wait >/dev/null
wait_ready

echo "rollback gate passed"
