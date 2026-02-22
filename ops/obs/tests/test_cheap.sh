#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"

OUT_DIR="$ROOT/artifacts/ops/obs"
mkdir -p "$OUT_DIR"

# Cheap observability evidence: scrape metrics + sample traces + validate log schema.
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/snapshot_metrics.py" "$OUT_DIR"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/snapshot_traces.py" "$OUT_DIR"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/validate_logs_schema.py" --file "$ROOT/ops/obs/contract/logs.example.jsonl"

# Keep cheap gate deterministic and contract-backed.
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_metrics_contract.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_trace_golden.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_dashboard_metric_compat.py"
python3 "$ROOT/ops/obs/scripts/areas/contracts/check_alerts_contract.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/log_schema_violation_injection.py"

echo "observability cheap suite passed"
