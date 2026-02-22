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
need curl
: "${ATLAS_E2E_ENABLE_OTEL:=0}"
if [ "$ATLAS_E2E_ENABLE_OTEL" != "1" ]; then
  echo "otel disabled; skip"
  exit 0
fi
install_chart
wait_ready
with_port_forward 18080
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py" otel-outage
echo "otel outage drill passed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
