#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need helm

helm template "$RELEASE" "$CHART" -n "$NS" -f "$VALUES" > /tmp/rbac-audit.yaml
if grep -Eq '^kind: (Role|RoleBinding|ClusterRole|ClusterRoleBinding|ServiceAccount)$' /tmp/rbac-audit.yaml; then
  echo "rbac minimalism violated: unexpected RBAC objects rendered" >&2
  exit 1
fi

echo "rbac minimalism gate passed"
