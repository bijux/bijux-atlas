#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need curl

install_chart
wait_ready
with_port_forward 18080

(cd "$ROOT" && ./bin/atlasctl ops load --report text run --suite pod-churn.json --out artifacts/perf/results) >/dev/null &
load_pid=$!
"$ROOT/bin/atlasctl" run ./packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/checks/rollout/pod_churn.py
wait "$load_pid"

curl -fsS "$BASE_URL/healthz" >/dev/null || {
  echo "failure_mode: pod_churn_healthz_unavailable" >&2
  exit 1
}
echo "pod churn under load contract passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
