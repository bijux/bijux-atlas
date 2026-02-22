#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm; need kubectl; need curl
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
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
