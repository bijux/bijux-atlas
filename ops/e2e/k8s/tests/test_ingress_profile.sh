#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need helm

PROFILE="$ROOT/ops/k8s/values/ingress.yaml"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$PROFILE" > /tmp/ingress-profile.yaml
grep -q "kind: Ingress" /tmp/ingress-profile.yaml
grep -q "host: atlas.local" /tmp/ingress-profile.yaml

echo "ingress profile gate passed"
