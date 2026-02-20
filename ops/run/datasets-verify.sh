#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-datasets-verify"
ops_version_guard python3
./ops/datasets/scripts/suite.sh verify
./ops/datasets/scripts/py/catalog_validate.py
