from __future__ import annotations

import os
import sys
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import env_root, sh


SCRIPT = r"""
ROOT="${ATLAS_REPO_ROOT}"
. "$ROOT/ops/k8s/tests/checks/_lib/k8s-suite-lib.sh"
need helm; need kubectl; need curl

python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/run_two_release_diff.py"

BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
Q="/v1/diff/genes?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&limit=10"
BASELINE="$(curl -fsS "$BASE_URL$Q")"
REV1="$(helm -n "$NS" history "$RELEASE" -o json | grep -o '"revision":[0-9]*' | tail -n1 | cut -d: -f2)"

errors_file="$(mktemp)"
(
  i=0
  while [ $i -lt 140 ]; do
    if ! curl -fsS "$BASE_URL/healthz" >/dev/null; then
      echo "healthz" >> "$errors_file"
    fi
    if ! curl -fsS "$BASE_URL$Q" >/dev/null; then
      echo "diff" >> "$errors_file"
    fi
    i=$((i+1))
    sleep 0.5
  done
) &
probe_pid=$!

helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" --set server.responseMaxBytes=262144 >/dev/null
helm rollback "$RELEASE" "$REV1" -n "$NS" --wait >/dev/null
wait "$probe_pid"

if [ -s "$errors_file" ]; then
  echo "rollback drill had request failures:" >&2
  cat "$errors_file" >&2
  exit 1
fi

POST="$(curl -fsS "$BASE_URL$Q")"
python3 - <<'PY' "$BASELINE" "$POST"
import json,sys
b=json.loads(sys.argv[1]); p=json.loads(sys.argv[2])
assert b.get("diff",{}).get("rows") == p.get("diff",{}).get("rows"), "semantic drift after rollback"
PY

echo "rollback drill passed"
"""


def main() -> int:
    root = env_root()
    env = os.environ.copy()
    env["ATLAS_REPO_ROOT"] = str(root)
    sh(["bash", "-ceu", SCRIPT], env=env)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
