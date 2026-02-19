#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

install_chart
wait_ready

ROOT="${ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)}"
. "$ROOT/ops/stack/tests/minio-invariants.sh"
check_minio_reachable_from_atlas "${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
