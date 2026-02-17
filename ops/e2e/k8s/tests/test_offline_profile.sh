#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$(dirname "$0")/common.sh"
setup_test_traps

OFFLINE="$ROOT/ops/k8s/values/offline.yaml"
install_chart -f "$OFFLINE"
wait_ready

echo "offline profile gate passed"
