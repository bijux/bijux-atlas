#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl; need helm

install_chart
wait_ready
helm -n "$NS" uninstall "$RELEASE" >/dev/null
sleep 3
kubectl -n "$NS" get deploy "$SERVICE_NAME" >/dev/null 2>&1 && {
  echo "deployment still exists after uninstall" >&2
  exit 1
}
install_chart
wait_ready

echo "uninstall reinstall gate passed"
