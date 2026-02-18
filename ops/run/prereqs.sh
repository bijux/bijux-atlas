#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-prereqs"
for c in docker kind kubectl helm k6 python3; do
  command -v "$c" >/dev/null 2>&1 || ops_fail "$OPS_ERR_PREREQ" "missing required tool: $c"
done
python3 --version
kubectl version --client >/dev/null
helm version --short >/dev/null
kind version >/dev/null
k6 version >/dev/null
python3 ./scripts/layout/check_tool_versions.py kind kubectl helm k6 >/dev/null || ops_fail "$OPS_ERR_VERSION" "tool version mismatch"
