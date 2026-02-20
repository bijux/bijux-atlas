#!/usr/bin/env bash
# Purpose: initialize deterministic ops run identity (run-id, namespace, artifact dir).
# Inputs: optional OPS_RUN_ID/OPS_NAMESPACE/OPS_RUN_DIR env vars.
# Outputs: exported OPS_RUN_ID, OPS_NAMESPACE, OPS_RUN_DIR, ATLAS_NS, ATLAS_E2E_NAMESPACE.
set -euo pipefail

ops_init_run_id() {
  if [ -z "${OPS_RUN_ID:-}" ]; then
    OPS_RUN_ID="atlas-ops-$(date -u +%Y%m%d-%H%M%S)"
  fi
  OPS_NAMESPACE="${OPS_NAMESPACE:-$OPS_RUN_ID}"
  OPS_RUN_DIR="${OPS_RUN_DIR:-$REPO_ROOT/artifacts/ops/$OPS_RUN_ID}"

  export OPS_RUN_ID OPS_NAMESPACE OPS_RUN_DIR
  export ATLAS_NS="${ATLAS_NS:-$OPS_NAMESPACE}"
  export ATLAS_E2E_NAMESPACE="${ATLAS_E2E_NAMESPACE:-$OPS_NAMESPACE}"

  mkdir -p "$OPS_RUN_DIR"
}

ops_require_ci_noninteractive() {
  if [ -n "${CI:-}" ] && [ "${OPS_ALLOW_PROMPT:-0}" = "1" ]; then
    echo "interactive prompts are forbidden in CI ops runs" >&2
    return 1
  fi
}
