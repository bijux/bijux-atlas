#!/usr/bin/env bash
# Purpose: canonical artifact path helpers for ops scripts.
# Inputs: optional OPS_RUN_ID/OPS_RUN_DIR env vars.
# Outputs: deterministic directories under artifacts/ops/<run-id>/.
set -euo pipefail

OPS_LIB_ROOT="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(CDPATH='' cd -- "${OPS_LIB_ROOT}/.." && pwd)"

ops_run_id() {
  if [ -n "${OPS_RUN_ID:-}" ]; then
    printf '%s\n' "$OPS_RUN_ID"
  elif [ -n "${ATLAS_RUN_ID:-}" ]; then
    printf '%s\n' "$ATLAS_RUN_ID"
  else
    printf '%s\n' "local"
  fi
}

ops_run_dir() {
  if [ -n "${OPS_RUN_DIR:-}" ]; then
    printf '%s\n' "$OPS_RUN_DIR"
  else
    printf '%s\n' "${REPO_ROOT}/artifacts/ops/$(ops_run_id)"
  fi
}

ops_artifact_dir() {
  local component="$1"
  local out
  out="$(ops_run_dir)/$component"
  mkdir -p "$out"
  printf '%s\n' "$out"
}
