#!/usr/bin/env python3
from __future__ import annotations
from pathlib import Path
from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm

if helm template "$RELEASE" "$CHART" -n "$NS" -f "$ROOT/ops/k8s/values/perf.yaml" \
  --set hpa.enabled=true \
  --set metrics.customMetrics.enabled=false >/tmp/atlas-hpa-misconfig.yaml 2>/tmp/atlas-hpa-misconfig.err; then
  echo "expected helm template to fail when hpa.enabled=true and metrics.customMetrics.enabled=false" >&2
  exit 1
fi

grep -Eq "hpa.enabled=true requires metrics.customMetrics.enabled=true|metrics.customMetrics.enabled" /tmp/atlas-hpa-misconfig.err
echo "hpa misconfig negative check passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
