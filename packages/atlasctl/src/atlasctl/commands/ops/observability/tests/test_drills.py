#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    script = r"""
set -euo pipefail
ROOT="$(pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"
require_bin python3

DRILLS=()
while IFS= read -r line; do
  [ -n "$line" ] && DRILLS+=("$line")
done <<EOF
$(python3 - <<'PY'
import json
for d in json.load(open('ops/obs/drills/drills.json')).get('drills', []):
    print(d['name'])
PY
)
EOF

for drill in "${DRILLS[@]}"; do
  echo "running drill from manifest: $drill"
  python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py" "$drill"
done
"$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/summarize_drill_results.py"
test -s "$ROOT/artifacts/observability/drill-conformance-report.json"

echo "observability drill manifest run passed"
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
