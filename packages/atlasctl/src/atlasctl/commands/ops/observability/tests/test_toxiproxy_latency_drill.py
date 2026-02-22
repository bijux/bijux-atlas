#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    script = r"""
set -euo pipefail
. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/assets/k8s_test_common.sh"
setup_test_traps
need kubectl
: "${ATLAS_E2E_ENABLE_TOXIPROXY:=0}"
if [ "$ATLAS_E2E_ENABLE_TOXIPROXY" != "1" ]; then
  echo "toxiproxy disabled; skip"
  exit 0
fi
install_chart
wait_ready
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py" store-latency-injection
echo "toxiproxy latency drill test passed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
