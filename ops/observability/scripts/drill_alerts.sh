#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
python3 "$ROOT/scripts/observability/check_alerts_contract.py"
echo "alert drill contract check passed"
