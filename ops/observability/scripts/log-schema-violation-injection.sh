#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: inject invalid JSON log line and assert log schema validator rejects it.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
out="${ROOT}/artifacts/observability/drills/log-schema-violation.jsonl"
mkdir -p "$(dirname "$out")"
echo '{"event":"request_end","request_id":123,"dataset":null}' > "$out"
if python3 "$ROOT/ops/observability/scripts/validate_logs_schema.py" --file "$out" >/dev/null 2>&1; then
  echo "expected log schema validator to fail" >&2
  exit 1
fi
echo "log schema violation injection drill passed"
