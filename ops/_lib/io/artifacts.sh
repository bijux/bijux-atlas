#!/usr/bin/env bash
# Purpose: canonical artifact path helpers for ops scripts.
# Inputs: optional OPS_RUN_ID/OPS_RUN_DIR env vars.
# Outputs: deterministic directories under artifacts/ops/<run-id>/.
set -euo pipefail

_OPS_LIB_DIR="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(CDPATH='' cd -- "${_OPS_LIB_DIR}/../../.." && pwd)"
# shellcheck source=ops/_lib/io/run_id.sh
source "${_OPS_LIB_DIR}/run_id.sh"

ops_run_id() {
  ops_init_run_id
  if [ -n "${OPS_RUN_ID:-}" ]; then
    printf '%s\n' "$OPS_RUN_ID"
  elif [ -n "${ATLAS_RUN_ID:-}" ]; then
    printf '%s\n' "$ATLAS_RUN_ID"
  else
    printf '%s\n' "local"
  fi
}

ops_run_dir() {
  ops_init_run_id
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

_ops_sha256() {
  if command -v sha256sum >/dev/null 2>&1; then
    sha256sum "$1" | awk '{print $1}'
  else
    shasum -a 256 "$1" | awk '{print $1}'
  fi
}

ops_write_metadata() {
  ops_init_run_id
  local out="${1:-$(ops_run_dir)}"
  mkdir -p "$out"
  local git_sha image_digest policy_hash dataset_hash tools_json
  git_sha="$(git -C "$REPO_ROOT" rev-parse --short HEAD 2>/dev/null || echo unknown)"
  image_digest="$(docker image inspect --format '{{index .RepoDigests 0}}' bijux-atlas:local 2>/dev/null || echo unknown)"
  policy_hash="$(_ops_sha256 "$REPO_ROOT/configs/policy/policy.json" 2>/dev/null || echo unknown)"
  dataset_hash="$(_ops_sha256 "$REPO_ROOT/ops/fixtures/medium/v1/manifest.lock" 2>/dev/null || echo unknown)"
  tools_json="$(cat "$REPO_ROOT/configs/ops/tool-versions.json" 2>/dev/null || echo '{}')"
  {
    echo '{'
    echo "  \"run_id\": \"${OPS_RUN_ID}\","
    echo "  \"namespace\": \"${OPS_NAMESPACE}\","
    echo "  \"git_sha\": \"${git_sha}\","
    echo "  \"image_digest\": \"${image_digest}\","
    echo "  \"policy_hash\": \"${policy_hash}\","
    echo "  \"dataset_hash\": \"${dataset_hash}\","
    echo "  \"tool_versions\": ${tools_json}"
    echo '}'
  } > "$out/metadata.json"
}
