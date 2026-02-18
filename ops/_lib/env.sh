#!/usr/bin/env bash
# Purpose: canonical ops env loader from env schema contract.
set -euo pipefail

ops_env_load() {
  python3 ./scripts/layout/validate_ops_env.py --schema "${OPS_ENV_SCHEMA:-configs/ops/env.schema.json}" >/dev/null
  export RUN_ID="${RUN_ID:-${OPS_RUN_ID:-}}"
  export ARTIFACT_DIR="${ARTIFACT_DIR:-${OPS_RUN_DIR:-}}"
  if [ -z "${RUN_ID:-}" ] || [ -z "${ARTIFACT_DIR:-}" ]; then
    echo "RUN_ID and ARTIFACT_DIR must be set" >&2
    return 2
  fi
}
