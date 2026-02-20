#!/usr/bin/env bash
# Purpose: helm wrappers with retry and debug dumps on failure.
# Inputs: namespace/release plus helm args.
# Outputs: helm command output; manifest and k8s diagnostics on failure.
set -euo pipefail

OPS_LIB_ROOT="$(CDPATH='' cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd)"
# shellcheck source=ops/_lib/artifacts.sh
source "$OPS_LIB_ROOT/../artifacts.sh"
# shellcheck source=ops/_lib/kubectl.sh
source "$OPS_LIB_ROOT/../kubectl.sh"

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
