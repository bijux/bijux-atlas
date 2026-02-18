#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-ci-fast"
exec make ops-up ops-deploy ops-smoke ops-k8s-tests ops-load-smoke ops-observability-validate
