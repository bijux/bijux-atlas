#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm grep

PROFILE="$ROOT/ops/k8s/values/perf.yaml"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$PROFILE" > /tmp/perf-pin.yaml
grep -Eq 'image: ".+@sha256:[a-f0-9]{64}"' /tmp/perf-pin.yaml

echo "image digest pinning gate passed"
