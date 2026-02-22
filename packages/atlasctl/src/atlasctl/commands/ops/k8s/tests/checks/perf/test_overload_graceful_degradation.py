#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need curl

wait_ready
with_port_forward 18080
(cd "$ROOT" && ./bin/atlasctl ops load --report text run --suite spike-overload-proof.json --out artifacts/perf/results) >/dev/null

code="$(curl -s -o /tmp/atlas-overload-body.json -w '%{http_code}' \
  "$BASE_URL/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-999999999&limit=500" || true)"
case "$code" in
  200|422|429|503) ;;
  *)
    echo "failure_mode: overload_unexpected_status" >&2
    exit 1
    ;;
esac

if [ "$code" = "429" ] || [ "$code" = "503" ]; then
  grep -Eq '"code"\\s*:\\s*"(RateLimited|QueryRejectedByPolicy|NotReady)"' /tmp/atlas-overload-body.json || {
    echo "failure_mode: overload_missing_policy_code" >&2
    exit 1
  }
fi
echo "overload graceful degradation contract passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
