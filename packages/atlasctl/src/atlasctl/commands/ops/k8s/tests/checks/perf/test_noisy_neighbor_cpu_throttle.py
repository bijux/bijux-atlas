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
(cd "$ROOT" && ./bin/atlasctl ops load --report text run --suite noisy-neighbor-cpu-throttle.json --out artifacts/perf/results) >/dev/null
curl -fsS "$BASE_URL/healthz" >/dev/null || {
  echo "failure_mode: noisy_neighbor_healthz_unavailable" >&2
  exit 1
}
echo "noisy neighbor cpu throttle contract passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
