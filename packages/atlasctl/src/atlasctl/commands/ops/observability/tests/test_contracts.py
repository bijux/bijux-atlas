#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    script = r"""
set -euo pipefail
ROOT="$(pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"

python3 "$ROOT/packages/atlasctl/src/atlasctl/obs/contracts/check_metrics_contract.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/obs/contracts/check_dashboard_contract.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_dashboard_metric_compat.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_obs_budgets.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/obs/contracts/check_alerts_contract.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_endpoint_metrics_coverage.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_endpoint_trace_coverage.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_overload_behavior_contract.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/obs/contracts/check_profile_goldens.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/obs/contracts/lint_runbooks.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/validate_logs_schema.py" --file "$ROOT/ops/obs/contract/logs.example.jsonl"
"$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/check_pack_versions.py"

echo "observability pack contracts passed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
