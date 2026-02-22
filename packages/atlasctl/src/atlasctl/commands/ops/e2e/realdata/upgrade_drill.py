from __future__ import annotations

import os
import sys
from pathlib import Path

if __package__ in (None, ""):
    sys.path.insert(0, str(Path(__file__).resolve().parents[6]))

from atlasctl.commands.ops.e2e.realdata._common import env_root, sh


SCRIPT = r"""
ROOT="${ATLAS_REPO_ROOT}"
. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/assets/k8s_test_common.sh"
need helm; need kubectl; need curl

python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/run_two_release_diff.py"

BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
Q="/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1"
BEFORE="$(curl -fsS "$BASE_URL$Q")"

errors_file="$(mktemp)"
(
  i=0
  while [ $i -lt 120 ]; do
    if ! curl -fsS "$BASE_URL/healthz" >/dev/null; then
      echo "healthz" >> "$errors_file"
    fi
    if ! curl -fsS "$BASE_URL$Q" >/dev/null; then
      echo "genes" >> "$errors_file"
    fi
    i=$((i+1))
    sleep 0.5
  done
) &
probe_pid=$!

helm upgrade "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" \
  --set server.responseMaxBytes=393216 \
  --set server.requestTimeoutMs=5500 \
  --wait >/dev/null

wait "$probe_pid"

if [ -s "$errors_file" ]; then
  echo "upgrade drill had request failures:" >&2
  cat "$errors_file" >&2
  exit 1
fi

AFTER="$(curl -fsS "$BASE_URL$Q")"
python3 - <<'PY' "$BEFORE" "$AFTER"
import json,sys
b=json.loads(sys.argv[1]); a=json.loads(sys.argv[2])
assert b.get("rows") == a.get("rows"), "semantic drift after upgrade"
PY

echo "upgrade drill passed"
"""


def main() -> int:
    root = env_root()
    env = os.environ.copy()
    env["ATLAS_REPO_ROOT"] = str(root)
    sh(["bash", "-ceu", SCRIPT], env=env)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
