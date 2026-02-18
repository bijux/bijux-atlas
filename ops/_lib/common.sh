#!/usr/bin/env bash
# Purpose: shared ops shell helpers for retries, timeouts, wrappers, and artifact capture.
# Inputs: sourced by ops scripts.
# Outputs: utility functions and canonical repo paths.
set -euo pipefail

OPS_LIB_ROOT="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(CDPATH='' cd -- "${OPS_LIB_ROOT}/../.." && pwd)"
# shellcheck source=ops/_lib/run_id.sh
source "${OPS_LIB_ROOT}/run_id.sh"
# shellcheck source=ops/_lib/artifacts.sh
source "${OPS_LIB_ROOT}/artifacts.sh"
# shellcheck source=ops/_lib/retry.sh
source "${OPS_LIB_ROOT}/retry.sh"
# shellcheck source=ops/_lib/timeout.sh
source "${OPS_LIB_ROOT}/timeout.sh"
# shellcheck source=ops/_lib/trap_bundle.sh
source "${OPS_LIB_ROOT}/trap_bundle.sh"
# shellcheck source=ops/_lib/kubectl.sh
source "${OPS_LIB_ROOT}/kubectl.sh"
# shellcheck source=ops/_lib/helm.sh
source "${OPS_LIB_ROOT}/helm.sh"
ARTIFACTS_ROOT="${REPO_ROOT}/artifacts/ops"

ops_need_cmd() {
  command -v "$1" >/dev/null 2>&1 || {
    echo "required command not found: $1" >&2
    return 1
  }
}

ops_mkdir_artifacts() {
  mkdir -p "$ARTIFACTS_ROOT"
}

ops_kubectl_wait_condition() {
  local namespace="$1"
  local kind="$2"
  local name="$3"
  local condition="$4"
  local timeout_value="${5:-120s}"
  ops_kubectl_retry -n "$namespace" wait --for="condition=${condition}" --timeout="$timeout_value" "${kind}/${name}" >/dev/null
}

ops_capture_artifacts() {
  local namespace="$1"
  local release="$2"
  local out_dir="$3"
  mkdir -p "$out_dir"
  kubectl get ns "$namespace" > "$out_dir/ns.txt" 2>/dev/null || true
  kubectl -n "$namespace" get all -o wide > "$out_dir/all.txt" 2>/dev/null || true
  kubectl -n "$namespace" get events --sort-by=.lastTimestamp > "$out_dir/events.txt" 2>/dev/null || true
  kubectl -n "$namespace" logs -l app.kubernetes.io/instance="$release" --all-containers --tail=2000 > "$out_dir/logs.txt" 2>/dev/null || true
  helm -n "$namespace" get manifest "$release" > "$out_dir/helm-manifest.yaml" 2>/dev/null || true
}

ops_wait_namespace_termination() {
  local namespace="$1"
  local timeout_secs="${2:-120}"
  local waited=0
  if ! ops_kubectl get ns "$namespace" >/dev/null 2>&1; then
    return 0
  fi
  if [ -z "$(ops_kubectl get ns "$namespace" -o jsonpath='{.metadata.deletionTimestamp}' 2>/dev/null)" ]; then
    return 0
  fi
  echo "namespace $namespace is terminating; waiting up to ${timeout_secs}s..."
  while [ "$waited" -lt "$timeout_secs" ]; do
    if ! ops_kubectl get ns "$namespace" >/dev/null 2>&1; then
      return 0
    fi
    sleep 5
    waited=$((waited + 5))
  done
  return 1
}

ops_ci_no_prompt_policy() {
  ops_require_ci_noninteractive
}
