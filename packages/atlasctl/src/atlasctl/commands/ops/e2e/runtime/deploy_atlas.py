from __future__ import annotations

import json
from pathlib import Path
import subprocess

def _repo_root() -> Path:
    return Path(__file__).resolve().parents[7]


def _load_layer_defaults(root: Path) -> tuple[str, str, str]:
    contract = json.loads((root / "ops/_meta/layer-contract.json").read_text(encoding="utf-8"))
    namespaces = contract.get("namespaces", {}) if isinstance(contract, dict) else {}
    release_meta = contract.get("release_metadata", {}) if isinstance(contract, dict) else {}
    defaults = release_meta.get("defaults", {}) if isinstance(release_meta, dict) else {}
    services = contract.get("services", {}) if isinstance(contract, dict) else {}
    atlas = services.get("atlas", {}) if isinstance(services, dict) else {}
    ns_e2e = str(namespaces.get("e2e", "atlas-e2e"))
    release_name = str(defaults.get("release_name", "atlas-e2e"))
    service_name = str(atlas.get("service_name", "bijux-atlas"))
    return ns_e2e, release_name, service_name


def _script(default_ns_e2e: str, default_release: str, default_service: str) -> str:
    template = r'''set -euo pipefail
ROOT="$(pwd)"
DEFAULT_E2E_NAMESPACE="__DEFAULT_E2E_NAMESPACE__"
DEFAULT_RELEASE_NAME="__DEFAULT_RELEASE_NAME__"
DEFAULT_ATLAS_SERVICE_NAME="__DEFAULT_ATLAS_SERVICE_NAME__"
ops_artifact_dir() {
  local component="$1"
  local run_id="${OPS_RUN_ID:-${RUN_ID:-local}}"
  local out="${OPS_RUN_DIR:-$ROOT/artifacts/ops/$run_id}/$component"
  mkdir -p "$out"
  printf '%s\n' "$out"
}
ops_kubectl() { kubectl "$@"; }
ops_kubectl_retry() {
  local attempts="${OPS_KUBECTL_RETRIES:-3}"; local sleep_secs="${OPS_KUBECTL_RETRY_SLEEP_SECS:-2}"; local i=1
  while true; do ops_kubectl "$@" && return 0; [ "$i" -ge "$attempts" ] && return 1; i=$((i+1)); sleep "$sleep_secs"; done
}
ops_kubectl_dump_bundle() {
  local ns="${1:-${ATLAS_E2E_NAMESPACE:-atlas-e2e}}"; local out="${2:-$(ops_artifact_dir failure-bundle)}"; mkdir -p "$out"
  kubectl get pods -A -o wide > "$out/pods.txt" 2>/dev/null || true
  kubectl -n "$ns" get all -o wide > "$out/all-$ns.txt" 2>/dev/null || true
  kubectl get events -A --sort-by=.lastTimestamp > "$out/events.txt" 2>/dev/null || true
}
ops_wait_namespace_termination() {
  local namespace="$1"; local timeout_secs="${2:-120}"; local waited=0
  if ! ops_kubectl get ns "$namespace" >/dev/null 2>&1; then return 0; fi
  if [ -z "$(ops_kubectl get ns "$namespace" -o jsonpath='{.metadata.deletionTimestamp}' 2>/dev/null)" ]; then return 0; fi
  while [ "$waited" -lt "$timeout_secs" ]; do
    ! ops_kubectl get ns "$namespace" >/dev/null 2>&1 && return 0
    sleep 5; waited=$((waited + 5))
  done
  return 1
}
ops_helm_retry() {
  local ns="$1"; local release="$2"; shift 2
  local attempts="${OPS_HELM_RETRIES:-3}"; local sleep_secs="${OPS_HELM_RETRY_SLEEP_SECS:-2}"; local i=1
  while true; do
    if helm "$@"; then return 0; fi
    [ "$i" -ge "$attempts" ] && return 1
    i=$((i+1)); sleep "$sleep_secs"
  done
}
if [ "${CI:-0}" = "1" ]; then export ATLASCTL_NONINTERACTIVE=1; fi
RUN_ID="${RUN_ID:-${OPS_RUN_ID:-}}"
ARTIFACT_DIR="${ARTIFACT_DIR:-${OPS_RUN_DIR:-}}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-$DEFAULT_RELEASE_NAME}"
NS="${ATLAS_E2E_NAMESPACE:-$DEFAULT_E2E_NAMESPACE}"
VALUES="${ATLAS_E2E_VALUES_FILE:-$ROOT/ops/k8s/values/local.yaml}"
CLUSTER_NAME="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
USE_LOCAL_IMAGE="${ATLAS_E2E_USE_LOCAL_IMAGE:-1}"
LOCAL_IMAGE_REF="${ATLAS_E2E_LOCAL_IMAGE:-bijux-atlas:local}"
HELM_TIMEOUT="${ATLAS_E2E_HELM_TIMEOUT:-5m}"
if [ "${OPS_DRY_RUN:-0}" = "1" ]; then echo "DRY-RUN deploy_atlas.py ns=$NS release=$RELEASE values=$VALUES"; exit 0; fi
for b in helm kubectl; do command -v "$b" >/dev/null 2>&1 || { echo "$b is required" >&2; exit 1; }; done
if [ "$USE_LOCAL_IMAGE" = "1" ]; then
  command -v docker >/dev/null 2>&1 || { echo "docker is required when ATLAS_E2E_USE_LOCAL_IMAGE=1" >&2; exit 1; }
  command -v kind >/dev/null 2>&1 || { echo "kind is required when ATLAS_E2E_USE_LOCAL_IMAGE=1" >&2; exit 1; }
fi
if ! ops_wait_namespace_termination "$NS" 120; then
  echo "namespace $NS is still terminating after timeout" >&2
  ops_kubectl_dump_bundle "$NS" "$(ops_artifact_dir failure-bundle)"
  exit 1
fi
ops_kubectl get ns "$NS" >/dev/null 2>&1 || ops_kubectl create ns "$NS"
cleanup_stale_nodeport_conflicts() {
  local node_port
  node_port="$(awk '/^[[:space:]]*nodePort:/ {print $2; exit}' "$VALUES" 2>/dev/null || true)"
  if [ -z "$node_port" ] || [ "$node_port" = "null" ]; then return 0; fi
  while read -r ns svc ports; do
    [ -z "$ns" ] && continue
    [ "$ns" = "$NS" ] && continue
    if echo "$ports" | tr ',' '\n' | grep -qx "$node_port"; then
      if [ "$svc" = "$DEFAULT_ATLAS_SERVICE_NAME" ] && [[ "$ns" == atlas-* ]]; then
        echo "removing stale NodePort owner: ${ns}/${svc} (port ${node_port})"
        helm -n "$ns" uninstall "$RELEASE" >/dev/null 2>&1 || true
        kubectl -n "$ns" delete svc "$svc" --ignore-not-found >/dev/null 2>&1 || true
      else
        echo "nodePort ${node_port} already allocated by ${ns}/${svc}; refusing destructive cleanup" >&2
        return 1
      fi
    fi
  done < <(kubectl get svc -A -o custom-columns=NS:.metadata.namespace,NAME:.metadata.name,NODEPORTS:.spec.ports[*].nodePort --no-headers)
}
cleanup_stale_nodeport_conflicts
EXTRA_SET_ARGS=()
if ! kubectl api-resources 2>/dev/null | grep -q "^servicemonitors"; then EXTRA_SET_ARGS+=(--set serviceMonitor.enabled=false --set alertRules.enabled=false); fi
if [ -n "${ATLAS_PINNED_DATASETS:-}" ]; then
  IFS=',' read -r -a pin_arr <<<"${ATLAS_PINNED_DATASETS}"
  i=0; for ds in "${pin_arr[@]}"; do ds="$(echo "$ds" | xargs)"; [ -z "$ds" ] && continue; EXTRA_SET_ARGS+=(--set-string "cache.pinnedDatasets[$i]=$ds"); i=$((i + 1)); done
fi
RENDER_DIR="$(ops_artifact_dir helm-render)"; mkdir -p "$RENDER_DIR"; cp -f "$VALUES" "$RENDER_DIR/values.input.yaml"; printf '%s\n' "${EXTRA_SET_ARGS[@]}" > "$RENDER_DIR/extra-set-args.txt"
render_chart() { helm template "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" --namespace "$NS" -f "$VALUES" "${EXTRA_SET_ARGS[@]}"; }
render_chart > "$RENDER_DIR/rendered-manifest.yaml" || { echo "failed to render helm template for release=$RELEASE ns=$NS" >&2; exit 1; }
if [ "$USE_LOCAL_IMAGE" = "1" ]; then
  docker image inspect "$LOCAL_IMAGE_REF" >/dev/null 2>&1 || docker build -t "$LOCAL_IMAGE_REF" -f "$ROOT/docker/Dockerfile" "$ROOT"
  kind load docker-image "$LOCAL_IMAGE_REF" --name "$CLUSTER_NAME"
  ops_helm_retry "$NS" "$RELEASE" upgrade --install "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" --namespace "$NS" -f "$VALUES" --atomic --wait --timeout "$HELM_TIMEOUT" "${EXTRA_SET_ARGS[@]}" --set image.repository="${LOCAL_IMAGE_REF%:*}" --set image.tag="${LOCAL_IMAGE_REF#*:}" --set image.pullPolicy=IfNotPresent
else
  ops_helm_retry "$NS" "$RELEASE" upgrade --install "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" --namespace "$NS" -f "$VALUES" --atomic --wait --timeout "$HELM_TIMEOUT" "${EXTRA_SET_ARGS[@]}"
fi

echo "atlas deployed: release=$RELEASE ns=$NS"'''
    return (
        template
        .replace("__DEFAULT_E2E_NAMESPACE__", default_ns_e2e.replace('"', '\\"'))
        .replace("__DEFAULT_RELEASE_NAME__", default_release.replace('"', '\\"'))
        .replace("__DEFAULT_ATLAS_SERVICE_NAME__", default_service.replace('"', '\\"'))
    )


def main() -> int:
    root = _repo_root()
    default_ns_e2e, default_release, default_service = _load_layer_defaults(root)
    script = _script(default_ns_e2e, default_release, default_service)
    return subprocess.call(["bash", "-lc", script], cwd=root)


if __name__ == "__main__":
    raise SystemExit(main())
