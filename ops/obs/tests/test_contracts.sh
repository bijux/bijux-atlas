#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"

python3 "$ROOT/packages/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/contracts/check_metrics_contract.py"
python3 "$ROOT/packages/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/contracts/check_dashboard_contract.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_dashboard_metric_compat.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_obs_budgets.py"
python3 "$ROOT/packages/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/contracts/check_alerts_contract.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_endpoint_metrics_coverage.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_endpoint_trace_coverage.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_overload_behavior_contract.py"
python3 "$ROOT/ops/obs/scripts/contracts/check_profile_goldens.py"
python3 "$ROOT/packages/bijux-atlas-scripts/src/bijux_atlas_scripts/obs/contracts/lint_runbooks.py"
python3 "$ROOT/ops/obs/scripts/validate_logs_schema.py" --file "$ROOT/ops/obs/contract/logs.example.jsonl"
"$ROOT/ops/obs/scripts/check_pack_versions.sh"

echo "observability pack contracts passed"
