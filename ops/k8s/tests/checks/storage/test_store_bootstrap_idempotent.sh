#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need kubectl

ROOT="${ROOT:-$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)}"
. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/stack/tests/assets/minio_invariants.sh"
check_minio_bootstrap_idempotent "$ROOT" "${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
