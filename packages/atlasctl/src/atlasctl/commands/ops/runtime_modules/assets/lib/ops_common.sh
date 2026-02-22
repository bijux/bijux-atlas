#!/usr/bin/env bash
# Purpose: shared ops shell helpers for retries, timeouts, wrappers, and artifact capture.
# Inputs: sourced by ops scripts.
# Outputs: utility functions and canonical repo paths.
set -euo pipefail

OPS_LIB_ROOT="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(CDPATH='' cd -- "${OPS_LIB_ROOT}/../.." && pwd)"
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

ops_layer_contract_file() {
  printf '%s\n' "${ATLAS_LAYER_CONTRACT_PATH:-$REPO_ROOT/ops/_meta/layer-contract.json}"
}

ops_layer_contract_get() {
  local key="$1"
  python3 - "$key" "$(ops_layer_contract_file)" <<'PY'
import json, sys
key = sys.argv[1]
path = sys.argv[2]
obj = json.load(open(path, encoding='utf-8'))
cur = obj
for part in key.split('.'):
    if isinstance(cur, dict) and part in cur:
        cur = cur[part]
    else:
        raise SystemExit(f"missing key: {key}")
if isinstance(cur, (dict, list)):
    print(json.dumps(cur, sort_keys=True))
else:
    print(cur)
PY
}

ops_layer_ns_stack() { ops_layer_contract_get "namespaces.stack"; }
ops_layer_ns_k8s() { ops_layer_contract_get "namespaces.k8s"; }
ops_layer_ns_e2e() { ops_layer_contract_get "namespaces.e2e"; }
ops_layer_service_atlas() { ops_layer_contract_get "services.atlas.service_name"; }
ops_layer_port_atlas() { ops_layer_contract_get "ports.atlas.service"; }
ops_layer_port_prometheus() { ops_layer_contract_get "ports.prometheus.service"; }
ops_layer_port_otel_grpc() { ops_layer_contract_get "ports.otel.grpc"; }
ops_layer_port_otel_http() { ops_layer_contract_get "ports.otel.http"; }
ops_layer_port_grafana() { ops_layer_contract_get "ports.grafana.service"; }
ops_layer_port_minio_api() { ops_layer_contract_get "ports.minio.api"; }
ops_layer_port_redis() { ops_layer_contract_get "ports.redis.service"; }

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

ops_context_guard() {
  local profile="${1:-kind}"
  if [ "${I_KNOW_WHAT_I_AM_DOING:-0}" = "1" ] || [ "${ALLOW_NON_KIND:-0}" = "1" ]; then
    return 0
  fi
  local ctx
  ctx="$(kubectl config current-context 2>/dev/null || true)"
  case "$profile" in
    kind)
      case "$ctx" in
        kind-*|*kind*) return 0 ;;
      esac
      echo "invalid kubectl context '$ctx' for profile=kind" >&2
      return 2
      ;;
    perf)
      if [ -z "$ctx" ]; then
        echo "missing kubectl context for profile=perf" >&2
        return 2
      fi
      return 0
      ;;
    *)
      echo "unknown profile '$profile'" >&2
      return 2
      ;;
  esac
}

ops_version_guard() {
  local tools=()
  if [ "$#" -gt 0 ]; then
    tools=("$@")
  else
    tools=(kind k6 helm kubectl jq yq)
  fi
  python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py "${tools[@]}"
}

ops_env_load() {
  python3 ./packages/atlasctl/src/atlasctl/layout_checks/validate_ops_env.py --schema "${OPS_ENV_SCHEMA:-configs/ops/env.schema.json}" >/dev/null
  export RUN_ID="${RUN_ID:-${OPS_RUN_ID:-}}"
  export ARTIFACT_DIR="${ARTIFACT_DIR:-${OPS_RUN_DIR:-}}"
  if [ -z "${RUN_ID:-}" ] || [ -z "${ARTIFACT_DIR:-}" ]; then
    echo "RUN_ID and ARTIFACT_DIR must be set" >&2
    return 2
  fi
}

ops_kubectl_guardrails() {
  if [ "${I_KNOW_WHAT_I_AM_DOING:-0}" = "1" ] || [ "${ALLOW_NON_KIND:-0}" = "1" ]; then
    return 0
  fi
  local ctx
  ctx="$(kubectl config current-context 2>/dev/null || true)"
  case "$ctx" in
    kind-*|*kind*) ;;
    *)
      echo "refusing kubectl command on non-kind context '$ctx' (set I_KNOW_WHAT_I_AM_DOING=1 to override)" >&2
      return 11
      ;;
  esac
}

ops_kubectl() {
  local timeout_secs="${OPS_KUBECTL_TIMEOUT_SECS:-120}"
  ops_kubectl_guardrails
  if command -v timeout >/dev/null 2>&1; then
    timeout "$timeout_secs" kubectl "$@"
  else
    kubectl "$@"
  fi
}

ops_kubectl_retry() {
  local attempts="${OPS_KUBECTL_RETRIES:-3}"
  local sleep_secs="${OPS_KUBECTL_RETRY_SLEEP_SECS:-2}"
  local i=1
  while true; do
    if ops_kubectl "$@"; then
      return 0
    fi
    if [ "$i" -ge "$attempts" ]; then
      return 1
    fi
    i=$((i + 1))
    sleep "$sleep_secs"
  done
}

ops_kubectl_dump_bundle() {
  local ns="${1:-${ATLAS_E2E_NAMESPACE:-atlas-e2e}}"
  local out="${2:-$(ops_artifact_dir failure-bundle)}"
  mkdir -p "$out"
  kubectl get pods -A -o wide > "$out/pods.txt" 2>/dev/null || true
  kubectl get all -A -o wide > "$out/all-cluster.txt" 2>/dev/null || true
  kubectl get events -A --sort-by=.lastTimestamp > "$out/events.txt" 2>/dev/null || true
  kubectl -n "$ns" get all -o wide > "$out/all-$ns.txt" 2>/dev/null || true
  kubectl -n "$ns" logs -l app.kubernetes.io/name=bijux-atlas --tail=2000 > "$out/logs-$ns.txt" 2>/dev/null || true
  kubectl -n kube-system get pods -o wide > "$out/kube-system-pods.txt" 2>/dev/null || true
  for p in $(kubectl -n kube-system get pods -o jsonpath='{.items[*].metadata.name}' 2>/dev/null || true); do
    kubectl -n kube-system logs "$p" --tail=500 > "$out/kube-system-$p.log" 2>/dev/null || true
  done
}

ops_helm() {
  helm "$@"
}

ops_helm_retry() {
  local ns="$1"
  local release="$2"
  shift 2
  local attempts="${OPS_HELM_RETRIES:-3}"
  local sleep_secs="${OPS_HELM_RETRY_SLEEP_SECS:-2}"
  local i=1
  while true; do
    if helm "$@"; then
      return 0
    fi
    if [ "$i" -ge "$attempts" ]; then
      local out
      out="$(ops_artifact_dir failure-bundle)"
      helm -n "$ns" get manifest "$release" > "$out/helm-manifest.yaml" 2>/dev/null || true
      helm -n "$ns" status "$release" > "$out/helm-status.txt" 2>/dev/null || true
      ops_kubectl_dump_bundle "$ns" "$out"
      return 1
    fi
    i=$((i + 1))
    sleep "$sleep_secs"
  done
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
