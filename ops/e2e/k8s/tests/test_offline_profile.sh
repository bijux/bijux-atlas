#!/usr/bin/env sh
set -eu
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$(dirname "$0")/common.sh"

OFFLINE="$ROOT/charts/bijux-atlas/values-offline.yaml"
install_chart -f "$OFFLINE"
wait_ready

echo "offline profile gate passed"
