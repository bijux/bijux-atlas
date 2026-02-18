#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-report"
out="${OPS_RUN_DIR:-ops/_artifacts/ops/manual}"
mkdir -p "$out"
OPS_RUN_DIR="$out" python3 - <<'PY'
import json
import os
from pathlib import Path
root=Path('.')
out=Path(os.environ.get('OPS_RUN_DIR','ops/_artifacts/ops/manual'))/'merged-report.json'
payload={
  'stack': str((root/'ops/_artifacts/stack-report/pass-fail-summary.json')),
  'obs': str((root/'ops/_artifacts/ops/obs/slo-burn.json')),
  'load': str((root/'ops/_artifacts/perf/results').as_posix()),
}
schema = json.loads((root/'ops/_schemas/report/unified.schema.json').read_text(encoding='utf-8'))
for key in schema.get('required', []):
  if key not in payload:
    raise SystemExit(f"merged report missing required key: {key}")
out.parent.mkdir(parents=True, exist_ok=True)
out.write_text(json.dumps(payload, indent=2)+"\n", encoding='utf-8')
print(out)
PY
