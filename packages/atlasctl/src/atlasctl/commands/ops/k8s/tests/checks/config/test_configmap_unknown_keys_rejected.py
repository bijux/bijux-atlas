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
CM_NAME="${SERVICE_NAME}-config"
kubectl -n "$NS" patch configmap "$CM_NAME" --type merge -p '{"data":{"ATLAS_UNKNOWN_KEY_SHOULD_FAIL":"1"}}' >/dev/null
if ./bin/atlasctl ops k8s --report text validate-configmap-keys "$NS" "$SERVICE_NAME"; then
  echo "unknown configmap key guard failed: validator accepted unexpected key" >&2
  exit 1
fi
kubectl -n "$NS" patch configmap "$CM_NAME" --type json -p='[{"op":"remove","path":"/data/ATLAS_UNKNOWN_KEY_SHOULD_FAIL"}]' >/dev/null
./bin/atlasctl ops k8s --report text validate-configmap-keys "$NS" "$SERVICE_NAME"
echo "configmap unknown key guard passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
