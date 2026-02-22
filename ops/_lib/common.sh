#!/usr/bin/env bash
# Purpose: shared ops shell helpers for retries, timeouts, wrappers, and artifact capture.
# Inputs: sourced by ops scripts.
# Outputs: utility functions and canonical repo paths.
set -euo pipefail

OPS_LIB_ROOT="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(CDPATH='' cd -- "${OPS_LIB_ROOT}/../.." && pwd)"
# shellcheck source=ops/_lib/io/artifacts.sh
source "${OPS_LIB_ROOT}/io/artifacts.sh"
# shellcheck source=ops/_lib/k8s/kubectl.sh
source "${OPS_LIB_ROOT}/k8s/kubectl.sh"
# shellcheck source=ops/_lib/helm/helm.sh
source "${OPS_LIB_ROOT}/helm/helm.sh"
# shellcheck source=ops/_lib/guard/context_guard.sh
source "${OPS_LIB_ROOT}/guard/context_guard.sh"
# shellcheck source=ops/_lib/guard/version_guard.sh
source "${OPS_LIB_ROOT}/guard/version_guard.sh"
# shellcheck source=ops/_lib/guard/env.sh
source "${OPS_LIB_ROOT}/guard/env.sh"
# shellcheck source=ops/_lib/report/layer_contract.sh
source "${OPS_LIB_ROOT}/report/layer_contract.sh"
ARTIFACTS_ROOT="${REPO_ROOT}/artifacts/ops"

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

ops_bundle_on_err() {
  local ns="$1"
  local release="$2"
  local out
  out="$(ops_artifact_dir failure-bundle)"
  ops_capture_artifacts "$ns" "$release" "$out" || true
  ops_kubectl_dump_bundle "$ns" "$out" || true
  echo "ops failure bundle: $out" >&2
}

ops_install_bundle_trap() {
  local ns="${1:-${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-atlas-e2e}}}"
  local release="${2:-${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}}"
  OPS_BUNDLE_TRAP_NS="$ns"
  OPS_BUNDLE_TRAP_RELEASE="$release"
  trap 'ops_bundle_on_err "$OPS_BUNDLE_TRAP_NS" "$OPS_BUNDLE_TRAP_RELEASE"' ERR
}

# Canonical ops exit codes (formerly sourced from log/errors.sh; inlined after shim removal).
OPS_ERR_CONFIG=10
OPS_ERR_CONTEXT=11
OPS_ERR_VERSION=12
OPS_ERR_PREREQ=13
OPS_ERR_TIMEOUT=14
OPS_ERR_VALIDATION=15
OPS_ERR_ARTIFACT=16
OPS_ERR_DOCS=17
OPS_ERR_INTERNAL=99

ops_fail() {
  local code="$1"
  shift
  printf '%s\n' "${*:-ops failure}" >&2
  exit "$code"
}

ops_log_json() {
  local level="$1"
  local event="$2"
  local msg="${3:-}"
  python3 - "$level" "$event" "$msg" <<'PY'
import json
import os
import sys
from datetime import datetime, timezone
level, event, msg = sys.argv[1], sys.argv[2], sys.argv[3]
print(json.dumps({
    "ts": datetime.now(timezone.utc).isoformat(),
    "level": level,
    "event": event,
    "msg": msg,
    "run_id": os.environ.get("RUN_ID") or os.environ.get("OPS_RUN_ID"),
    "artifact_dir": os.environ.get("ARTIFACT_DIR") or os.environ.get("OPS_RUN_DIR"),
}, separators=(",", ":")))
PY
}

ops_require_run_context() {
  local run_id="${RUN_ID:-${OPS_RUN_ID:-}}"
  local artifact_dir="${ARTIFACT_DIR:-${OPS_RUN_DIR:-}}"
  if [ -z "$run_id" ] || [ -z "$artifact_dir" ]; then
    echo "RUN_ID/OPS_RUN_ID and ARTIFACT_DIR/OPS_RUN_DIR are required" >&2
    return "$OPS_ERR_CONFIG"
  fi
}

ops_entrypoint_start() {
  local entrypoint="$1"
  ops_require_run_context || exit "$OPS_ERR_CONFIG"
  ops_install_bundle_trap
  ops_log_json "info" "entrypoint.start" "$entrypoint"
}

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
