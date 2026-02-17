#!/usr/bin/env bash
# Purpose: shared ops shell helpers for retries, timeouts, kubectl wrappers, and artifact capture.
# Inputs: sourced by ops scripts.
# Outputs: utility functions and canonical repo paths.
set -euo pipefail

OPS_LIB_ROOT="$(CDPATH= cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
REPO_ROOT="$(CDPATH= cd -- "${OPS_LIB_ROOT}/.." && pwd)"
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

ops_retry() {
  local attempts="$1"
  local sleep_seconds="$2"
  shift 2
  local n=1
  while true; do
    if "$@"; then
      return 0
    fi
    if [ "$n" -ge "$attempts" ]; then
      echo "command failed after $attempts attempts: $*" >&2
      return 1
    fi
    n=$((n + 1))
    sleep "$sleep_seconds"
  done
}

ops_timeout_run() {
  local timeout_seconds="$1"
  shift
  if command -v timeout >/dev/null 2>&1; then
    timeout "$timeout_seconds" "$@"
    return $?
  fi
  "$@"
}

ops_kubectl_wait_condition() {
  local namespace="$1"
  local kind="$2"
  local name="$3"
  local condition="$4"
  local timeout_value="${5:-120s}"
  kubectl -n "$namespace" wait --for="condition=${condition}" --timeout="$timeout_value" "${kind}/${name}" >/dev/null
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
