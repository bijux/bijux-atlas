#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"

python3 "$ROOT/scripts/public/observability/check_metrics_contract.py"
python3 "$ROOT/scripts/public/observability/check_dashboard_contract.py"
python3 "$ROOT/ops/obs/scripts/contracts/check_dashboard_metric_compat.py"
python3 "$ROOT/scripts/public/observability/check_alerts_contract.py"
python3 "$ROOT/scripts/public/observability/lint_runbooks.py"
python3 "$ROOT/ops/obs/scripts/validate_logs_schema.py" --file "$ROOT/ops/obs/contract/logs.example.jsonl"
"$ROOT/ops/obs/scripts/check_pack_versions.sh"

echo "observability pack contracts passed"
