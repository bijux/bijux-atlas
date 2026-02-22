#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    script = r"""
set -euo pipefail
. ops/obs/tests/common.sh
setup_test_traps
need kubectl

: "${ATLAS_E2E_ENABLE_OTEL:=0}"
if [ "$ATLAS_E2E_ENABLE_OTEL" != "1" ]; then
  echo "otel disabled; skip"
  exit 0
fi

install_chart
wait_ready
with_port_forward 18080
curl -fsS "$BASE_URL/healthz" >/dev/null
curl -fsS "$BASE_URL/readyz" >/dev/null || true
curl -fsS "$BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1" >/dev/null || true
pod=$(kubectl -n atlas-e2e get pod -l app=otel-collector -o jsonpath='{.items[0].metadata.name}')
kubectl -n atlas-e2e logs "$pod" --tail=500 | grep -E "dataset|query|serialize" >/dev/null

echo "otel span signal observed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
