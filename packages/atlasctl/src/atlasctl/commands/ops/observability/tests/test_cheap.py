#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    script = r"""
set -euo pipefail
ROOT="$(pwd)"
. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/tests/assets/observability_test_lib.sh"

OUT_DIR="$ROOT/artifacts/ops/obs"
mkdir -p "$OUT_DIR"

python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/snapshot_metrics.py" "$OUT_DIR"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/snapshot_traces.py" "$OUT_DIR"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/validate_logs_schema.py" --file "$ROOT/ops/obs/contract/logs.example.jsonl"

python3 "$ROOT/ops/obs/scripts/areas/contracts/check_metrics_contract.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_trace_golden.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_dashboard_metric_compat.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_alerts_contract.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/log_schema_violation_injection.py"

echo "observability cheap suite passed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
