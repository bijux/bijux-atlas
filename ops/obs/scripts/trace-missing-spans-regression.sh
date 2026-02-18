#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: inject trace snapshot missing required spans and ensure trace contract check fails.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
out="$ROOT/artifacts/ops/obs"
mkdir -p "$out"
printf '%s\n' '{"spans":[{"name":"request_root","request_id":"abc"}]}' > "$out/traces.snapshot.log"
printf '%s\n' '{"trace_id":"abc"}' > "$out/traces.exemplars.log"
if ATLAS_E2E_ENABLE_OTEL=1 python3 "$ROOT/ops/obs/scripts/contracts/check_trace_coverage.py" >/dev/null 2>&1; then
  echo "expected trace coverage check to fail for missing spans" >&2
  exit 1
fi
echo "trace missing spans regression drill passed"
