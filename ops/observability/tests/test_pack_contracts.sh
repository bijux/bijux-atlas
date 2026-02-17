#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/observability/tests/common.sh"

python3 "$ROOT/scripts/public/observability/check_metrics_contract.py"
python3 "$ROOT/scripts/public/observability/check_dashboard_contract.py"
python3 "$ROOT/scripts/public/observability/check_alerts_contract.py"
python3 "$ROOT/scripts/public/observability/lint_runbooks.py"

echo "observability pack contracts passed"
