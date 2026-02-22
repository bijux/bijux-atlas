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
release_stamp="$(kubectl -n "$NS" get configmap "$CM_NAME" -o jsonpath='{.data.ATLAS_CONFIG_RELEASE_ID}')"
schema_stamp="$(kubectl -n "$NS" get configmap "$CM_NAME" -o jsonpath='{.data.ATLAS_CONFIG_SCHEMA_VERSION}')"
[ -n "$release_stamp" ] || {
  echo "configmap version stamp check failed: ATLAS_CONFIG_RELEASE_ID is empty" >&2
  exit 1
}
[ -n "$schema_stamp" ] || {
  echo "configmap version stamp check failed: ATLAS_CONFIG_SCHEMA_VERSION is empty" >&2
  exit 1
}
echo "$release_stamp" | grep -Eq '.+:[0-9]+$' || {
  echo "configmap version stamp check failed: ATLAS_CONFIG_RELEASE_ID format expected <release>:<revision>, got: $release_stamp" >&2
  exit 1
}
echo "configmap version stamp check passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
