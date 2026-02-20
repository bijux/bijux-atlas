#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../../_lib/common.sh"
setup_test_traps
need helm shasum

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > /tmp/chart-a.yaml
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > /tmp/chart-b.yaml
A="$(shasum -a 256 /tmp/chart-a.yaml | awk '{print $1}')"
B="$(shasum -a 256 /tmp/chart-b.yaml | awk '{print $1}')"
[ "$A" = "$B" ] || { echo "chart drift detected across two renders" >&2; exit 1; }

echo "chart drift gate passed"
