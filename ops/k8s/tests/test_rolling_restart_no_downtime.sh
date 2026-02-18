#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl; need curl

install_chart
wait_ready
with_port_forward 18080

fail_file="$(mktemp)"
trap 'rm -f "$fail_file"' EXIT
echo 0 > "$fail_file"

(
  for _ in $(seq 1 60); do
    code="$(curl -s -o /dev/null -w '%{http_code}' "$BASE_URL/healthz" || true)"
    if [ "$code" != "200" ]; then
      n="$(cat "$fail_file")"
      echo $((n + 1)) > "$fail_file"
    fi
    sleep 1
  done
) &
probe_pid=$!

kubectl -n "$NS" rollout restart deployment/"$SERVICE_NAME" >/dev/null
kubectl -n "$NS" rollout status deployment/"$SERVICE_NAME" --timeout=180s >/dev/null
wait "$probe_pid"

failures="$(cat "$fail_file")"
[ "$failures" -le 2 ] || {
  echo "downtime detected during rolling restart, failures=$failures" >&2
  exit 1
}

echo "rolling restart no downtime gate passed"
