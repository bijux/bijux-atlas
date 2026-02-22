#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl
need curl

wait_ready
BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/overload_admission_control.py"

status="$(curl -s -o /tmp/atlas-admission-body.json -w '%{http_code}' \
  "$BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-999999999&limit=500")"

case "$status" in
  200|422|429|503) ;;
  *) echo "unexpected admission status: $status" >&2; exit 1 ;;
esac

if [ "$status" = "429" ] || [ "$status" = "503" ]; then
  if ! grep -Eq '"code"\\s*:\\s*"(RateLimited|QueryRejectedByPolicy|NotReady)"' /tmp/atlas-admission-body.json; then
    echo "expected policy/rejection code in admission response" >&2
    cat /tmp/atlas-admission-body.json >&2
    exit 1
  fi
fi

echo "admission control api behavior passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
