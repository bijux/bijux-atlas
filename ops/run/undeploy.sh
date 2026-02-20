#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-undeploy"
ops_version_guard helm

ns="${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-$(ops_layer_ns_e2e)}}"
release="${ATLAS_E2E_RELEASE_NAME:-$(ops_layer_contract_get release_metadata.defaults.release_name)}"
ops_helm -n "$ns" uninstall "$release" >/dev/null 2>&1 || true

echo "undeploy complete: release=$release namespace=$ns"
