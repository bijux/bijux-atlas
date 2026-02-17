#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need helm

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set nodeLocalSsdProfile.enabled=true | grep -q 'emptyDir:'

echo "node-local cache profile gate passed"
