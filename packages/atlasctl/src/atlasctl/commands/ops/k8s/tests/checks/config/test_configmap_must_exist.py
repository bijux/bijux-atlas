#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl
install_chart
wait_ready
cm="${SERVICE_NAME}-config"
kubectl -n "$NS" get configmap "$cm" >/dev/null
for key in ATLAS_CONFIG_RELEASE_ID ATLAS_CONFIG_SCHEMA_VERSION ATLAS_REQUEST_TIMEOUT_MS ATLAS_SQL_TIMEOUT_MS; do
  val="$(kubectl -n "$NS" get configmap "$cm" -o jsonpath="{.data.${key}}" || true)"
  [ -n "$val" ] || { echo "configmap must-exist check failed: missing key $key" >&2; exit 1; }
done
echo "configmap must exist contract passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
