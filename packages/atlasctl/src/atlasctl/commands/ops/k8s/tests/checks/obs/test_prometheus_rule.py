#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl

if ! kubectl api-resources | grep -q "^prometheusrules"; then
  echo "PrometheusRule CRD not present; skipping"
  exit 0
fi

install_chart --set serviceMonitor.enabled=true --set alertRules.enabled=true
wait_ready
kubectl -n "$NS" get prometheusrule "$SERVICE_NAME-alerts" >/dev/null

echo "prometheus rule gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
