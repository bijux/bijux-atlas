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

install_chart
wait_ready
out="${OPS_RUN_DIR:-ops/_artifacts/load/results}"
ATLAS_BASE_URL="$BASE_URL" ./bin/atlasctl ops load --report text run --suite noisy-neighbor-cpu-throttle.json --out "$out"

[ -f "$out/noisy-neighbor-cpu-throttle.json" ] || {
  echo "expected noisy-neighbor result missing" >&2
  exit 1
}

echo "cpu throttle noisy-neighbor scenario passed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
