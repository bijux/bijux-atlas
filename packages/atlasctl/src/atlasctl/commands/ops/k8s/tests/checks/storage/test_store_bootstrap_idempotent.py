#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need kubectl
ROOT="${ROOT:-$(pwd)}"
. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/stack/tests/assets/minio_invariants.sh"
check_minio_bootstrap_idempotent "$ROOT" "${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
