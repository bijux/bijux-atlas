#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-datasets-verify"
./ops/datasets/scripts/fetch_and_verify.sh
./ops/datasets/scripts/catalog_validate.py
