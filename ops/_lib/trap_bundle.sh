#!/usr/bin/env bash
# Purpose: install ERR trap to always collect failure bundle.
# Inputs: namespace, release (optional; defaults from env).
# Outputs: bundle artifacts under artifacts/ops/<run-id>/failure-bundle.
set -euo pipefail

ops_install_bundle_trap() {
  local ns="${1:-${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-atlas-e2e}}}"
  local release="${2:-${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}}"
  trap 'out="$(ops_artifact_dir failure-bundle)"; ops_capture_artifacts "'$ns'" "'$release'" "$out" || true; ops_kubectl_dump_bundle "'$ns'" "$out" || true; echo "ops failure bundle: $out" >&2' ERR
}
