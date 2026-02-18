#!/usr/bin/env bash
set -euo pipefail
. "$(dirname "$0")/common.sh"
setup_test_traps
need kubectl

. "$ROOT/ops/stack/tests/minio-invariants.sh"
check_minio_bootstrap_idempotent "$ROOT" "${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
