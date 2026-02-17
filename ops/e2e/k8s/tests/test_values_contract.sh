#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"
./scripts/contracts/check_chart_values_contract.py

echo "values contract gate passed"
