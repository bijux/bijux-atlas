#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set nodeLocalSsdProfile.enabled=true | grep -q 'emptyDir:'

echo "node-local cache profile gate passed"
