#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

install_chart
wait_ready
install_chart
wait_ready

echo "install twice idempotency passed"
