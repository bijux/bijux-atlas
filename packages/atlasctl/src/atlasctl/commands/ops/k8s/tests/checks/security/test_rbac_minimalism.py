#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

from ._shell_common import run_k8s_test_shell


def main() -> int:
    return run_k8s_test_shell(
        """
setup_test_traps
need helm
helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > /tmp/rbac-audit.yaml
if grep -Eq '^kind: (Role|RoleBinding|ClusterRole|ClusterRoleBinding|ServiceAccount)$' /tmp/rbac-audit.yaml; then
  echo "rbac minimalism violated: unexpected RBAC objects rendered" >&2
  exit 1
fi
echo "rbac minimalism gate passed"
        """,
        Path(__file__),
    )


if __name__ == "__main__":
    raise SystemExit(main())
