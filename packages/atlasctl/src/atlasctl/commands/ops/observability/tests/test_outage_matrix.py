#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    script = r"""
set -euo pipefail
ROOT="$(pwd)"
. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/tests/assets/observability_test_lib.sh"
require_bin python3

OUT_DIR="$ROOT/artifacts/observability"
OPS_OBS_DIR="$ROOT/artifacts/ops/obs"
mkdir -p "$OUT_DIR" "$OPS_OBS_DIR"

DRILLS=()
while IFS= read -r line; do
  [ -n "$line" ] && DRILLS+=("$line")
done <<EOF
$(python3 - <<'PY'
import json
for d in json.load(open('ops/obs/drills/drills.json')).get('drills', []):
    if d.get('outage_matrix'):
        print(d['name'])
PY
)
EOF

for drill in "${DRILLS[@]}"; do
  echo "running outage drill: $drill"
  python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py" "$drill"
done

cp -f "$OPS_OBS_DIR"/metrics.prom "$OUT_DIR"/metrics.prom 2>/dev/null || true
cp -f "$OPS_OBS_DIR"/traces.snapshot.log "$OUT_DIR"/traces.snapshot.log 2>/dev/null || true
cp -f "$OPS_OBS_DIR"/traces.exemplars.log "$OUT_DIR"/traces.exemplars.log 2>/dev/null || true
test -s "$OUT_DIR/metrics.prom"
test -s "$OUT_DIR/traces.snapshot.log"
test -s "$OUT_DIR/traces.exemplars.log"

echo "observability outage matrix passed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
