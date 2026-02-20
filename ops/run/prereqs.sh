#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-prereqs"
for c in docker kind kubectl helm k6 python3; do
  command -v "$c" >/dev/null 2>&1 || ops_fail "$OPS_ERR_PREREQ" "missing required tool: $c"
done
ops_version_guard kind kubectl helm k6
python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py kind kubectl helm k6 jq yq python3
python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_ops_pins.py
python3 --version
kubectl version --client >/dev/null
helm version --short >/dev/null
kind version >/dev/null
k6 version >/dev/null
