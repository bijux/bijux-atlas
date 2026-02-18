#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl; need curl

install_chart
wait_ready
with_port_forward 19090 atlas-e2e prometheus 9090
for _ in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:19090/api/v1/targets" | grep -q 'bijux-atlas.atlas-e2e.svc.cluster.local:8080'; then
    echo "prometheus scrape target present"
    exit 0
  fi
  sleep 2
done

echo "prometheus did not discover atlas scrape target" >&2
exit 1
