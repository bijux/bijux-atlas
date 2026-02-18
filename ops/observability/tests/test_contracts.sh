#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/observability/tests/observability-test-lib.sh"

python3 "$ROOT/scripts/public/observability/check_metrics_contract.py"
python3 "$ROOT/scripts/public/observability/check_dashboard_contract.py"
python3 "$ROOT/scripts/public/observability/check_alerts_contract.py"
python3 "$ROOT/scripts/public/observability/lint_runbooks.py"
python3 "$ROOT/ops/observability/scripts/validate_logs_schema.py" --file "$ROOT/ops/observability/contract/logs.example.jsonl"
"$ROOT/ops/observability/scripts/check_pack_versions.sh"

echo "observability pack contracts passed"
