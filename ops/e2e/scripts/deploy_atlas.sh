#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
source "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_install_bundle_trap "${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-atlas-e2e}}" "${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
ops_ci_no_prompt_policy
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
VALUES="${ATLAS_E2E_VALUES_FILE:-$ROOT/ops/k8s/values/local.yaml}"
CLUSTER_NAME="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
USE_LOCAL_IMAGE="${ATLAS_E2E_USE_LOCAL_IMAGE:-1}"
LOCAL_IMAGE_REF="${ATLAS_E2E_LOCAL_IMAGE:-bijux-atlas:local}"
HELM_TIMEOUT="${ATLAS_E2E_HELM_TIMEOUT:-5m}"

if [ "${OPS_DRY_RUN:-0}" = "1" ]; then
  echo "DRY-RUN deploy_atlas.sh ns=$NS release=$RELEASE values=$VALUES"
  exit 0
fi

if ! command -v helm >/dev/null 2>&1; then
  echo "helm is required" >&2
  exit 1
fi
if ! command -v kubectl >/dev/null 2>&1; then
  echo "kubectl is required" >&2
  exit 1
fi
if [ "$USE_LOCAL_IMAGE" = "1" ]; then
  if ! command -v docker >/dev/null 2>&1; then
    echo "docker is required when ATLAS_E2E_USE_LOCAL_IMAGE=1" >&2
    exit 1
  fi
  if ! command -v kind >/dev/null 2>&1; then
    echo "kind is required when ATLAS_E2E_USE_LOCAL_IMAGE=1" >&2
    exit 1
  fi
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
  if [ -z "$node_port" ] || [ "$node_port" = "null" ]; then
    return 0
  fi
  while read -r ns svc ports; do
    [ -z "$ns" ] && continue
    [ "$ns" = "$NS" ] && continue
    if echo "$ports" | tr ',' '\n' | grep -qx "$node_port"; then
      if [ "$svc" = "${RELEASE}-bijux-atlas" ] && [[ "$ns" == atlas-ops-* ]]; then
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
if ! kubectl api-resources 2>/dev/null | grep -q "^servicemonitors"; then
  EXTRA_SET_ARGS+=(--set serviceMonitor.enabled=false --set alertRules.enabled=false)
fi
if [ -n "${ATLAS_PINNED_DATASETS:-}" ]; then
  IFS=',' read -r -a pin_arr <<<"${ATLAS_PINNED_DATASETS}"
  i=0
  for ds in "${pin_arr[@]}"; do
    ds="$(echo "$ds" | xargs)"
    [ -z "$ds" ] && continue
    EXTRA_SET_ARGS+=(--set-string "cache.pinnedDatasets[$i]=$ds")
    i=$((i + 1))
  done
fi

RENDER_DIR="$(ops_artifact_dir helm-render)"
mkdir -p "$RENDER_DIR"
cp -f "$VALUES" "$RENDER_DIR/values.input.yaml"
printf '%s\n' "${EXTRA_SET_ARGS[@]}" > "$RENDER_DIR/extra-set-args.txt"

render_chart() {
  helm template "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" \
    --namespace "$NS" \
    -f "$VALUES" \
    "${EXTRA_SET_ARGS[@]}"
}

if ! render_chart > "$RENDER_DIR/rendered-manifest.yaml"; then
  echo "failed to render helm template for release=$RELEASE ns=$NS" >&2
  exit 1
fi

if [ "$USE_LOCAL_IMAGE" = "1" ]; then
  if ! docker image inspect "$LOCAL_IMAGE_REF" >/dev/null 2>&1; then
    docker build -t "$LOCAL_IMAGE_REF" -f "$ROOT/docker/Dockerfile" "$ROOT"
  fi
  kind load docker-image "$LOCAL_IMAGE_REF" --name "$CLUSTER_NAME"
  ops_helm_retry "$NS" "$RELEASE" upgrade --install "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" \
    --namespace "$NS" \
    -f "$VALUES" \
    --atomic --wait --timeout "$HELM_TIMEOUT" \
    "${EXTRA_SET_ARGS[@]}" \
    --set image.repository="${LOCAL_IMAGE_REF%:*}" \
    --set image.tag="${LOCAL_IMAGE_REF#*:}" \
    --set image.pullPolicy=IfNotPresent
else
  ops_helm_retry "$NS" "$RELEASE" upgrade --install "$RELEASE" "$ROOT/ops/k8s/charts/bijux-atlas" \
    --namespace "$NS" \
    -f "$VALUES" \
    --atomic --wait --timeout "$HELM_TIMEOUT" \
    "${EXTRA_SET_ARGS[@]}"
fi

echo "atlas deployed: release=$RELEASE ns=$NS"
