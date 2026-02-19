#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm kubectl curl

PROFILE="$ROOT/ops/k8s/values/ingress.yaml"
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" -f "$PROFILE" > /tmp/ingress-profile.yaml
grep -q "kind: Ingress" /tmp/ingress-profile.yaml
grep -q "host: atlas.local" /tmp/ingress-profile.yaml

if ! kubectl -n ingress-nginx get deploy ingress-nginx-controller >/dev/null 2>&1; then
  echo "ingress controller not present; render gate only"
  echo "ingress profile gate passed"
  exit 0
fi

install_chart -f "$PROFILE"
wait_ready
ip="$(kubectl -n "$NS" get ingress "$SERVICE_NAME" -o jsonpath='{.status.loadBalancer.ingress[0].ip}' || true)"
if [ -z "$ip" ]; then
  echo "ingress has no load balancer ip yet; skipping reachability probe"
  echo "ingress profile gate passed"
  exit 0
fi
curl -fsS -H 'Host: atlas.local' "http://$ip/healthz" >/dev/null

echo "ingress profile gate passed"
