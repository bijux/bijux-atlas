#!/usr/bin/env bash
set -euo pipefailo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need helm shasum

PROFILE="$ROOT/ops/k8s/values/multi-registry.yaml"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$PROFILE" > /tmp/multi-registry-a.yaml
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$PROFILE" > /tmp/multi-registry-b.yaml
shasum -a 256 /tmp/multi-registry-a.yaml /tmp/multi-registry-b.yaml | awk '{print $1}' | uniq | wc -l | grep -q '^1$'

echo "multi-registry deterministic render gate passed"
