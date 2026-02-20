#!/usr/bin/env bash
# Purpose: kubectl wrappers with retry/timeouts and failure bundle capture.
# Inputs: kubectl arguments and optional OPS_BUNDLE_DIR env var.
# Outputs: kubectl command output; bundle artifacts on failure.
set -euo pipefail

_OPS_LIB_DIR="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ops/_lib/artifacts.sh
source "$_OPS_LIB_DIR/../artifacts.sh"

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
