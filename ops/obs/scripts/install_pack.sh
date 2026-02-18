#!/usr/bin/env bash
set -euo pipefail
# shellcheck source=ops/_lib/common.sh
source "$(CDPATH= cd -- "$(dirname -- "$0")/../../_lib" && pwd)/common.sh"
# shellcheck source=ops/_lib/ports.sh
source "${REPO_ROOT}/ops/_lib/ports.sh"

PROFILE="${ATLAS_OBS_PROFILE:-kind}"
if [ "${1:-}" = "--profile" ]; then
  PROFILE="${2:-}"
fi
OBS_NS="${ATLAS_OBS_NAMESPACE:-atlas-observability}"
STORAGE_MODE="${ATLAS_OBS_STORAGE_MODE:-ephemeral}"
OFFLINE_MODE="${ATLAS_OBS_OFFLINE:-0}"

compose_cmd() {
  if command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1; then
    echo "docker compose"
    return 0
  fi
  if command -v docker-compose >/dev/null 2>&1; then
    echo "docker-compose"
    return 0
  fi
  return 1
}

ensure_offline_images() {
  python3 - <<'PY'
import json,subprocess,sys
cfg=json.load(open("configs/ops/obs-pack.json"))
for spec in cfg.get("images", {}).values():
    ref=spec["ref"]
    rc=subprocess.call(["docker","image","inspect",ref],stdout=subprocess.DEVNULL,stderr=subprocess.DEVNULL)
    if rc != 0:
        print(f"offline mode missing image locally: {ref}",file=sys.stderr)
        raise SystemExit(1)
print("offline image precheck passed")
PY
}

kind_load_offline_images() {
  local cluster_name="${ATLAS_E2E_CLUSTER_NAME:-bijux-atlas-e2e}"
  python3 - <<'PY' | while IFS= read -r ref; do
import json
cfg=json.load(open("configs/ops/obs-pack.json"))
for spec in cfg.get("images", {}).values():
    print(spec["ref"])
PY
    kind load docker-image "$ref" --name "$cluster_name"
  done
}

case "$PROFILE" in
  local-compose)
    cmd="$(compose_cmd)" || {
      echo "local-compose profile requires docker compose or docker-compose" >&2
      exit 1
    }
    if [ "$OFFLINE_MODE" = "1" ]; then
      ensure_offline_images
      $cmd -f "${REPO_ROOT}/ops/obs/pack/compose/docker-compose.yml" up -d --pull never
    else
      $cmd -f "${REPO_ROOT}/ops/obs/pack/compose/docker-compose.yml" up -d
    fi
    ;;
  kind | cluster)
    ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/namespace.yaml"
    ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/rbac.yaml"
    if [ "$OFFLINE_MODE" = "1" ]; then
      ensure_offline_images
      kind_load_offline_images
    fi
    ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/prometheus-config.yaml"
    if [ "$STORAGE_MODE" = "persistent" ]; then
      ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/prometheus-pvc.yaml"
      ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/grafana-pvc.yaml"
    fi
    ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/prometheus.yaml"
    ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/grafana-config.yaml"
    ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/grafana.yaml"
    kubectl -n "$OBS_NS" create configmap atlas-observability-otel-config \
      --from-file=config.yaml="${REPO_ROOT}/ops/obs/pack/otel/config.yaml" \
      --dry-run=client -o yaml | kubectl apply -f -
    ops_kubectl apply -f "${REPO_ROOT}/ops/obs/pack/k8s/otel.yaml"
    if ops_kubectl api-resources | grep -q "^prometheusrules"; then
      ops_kubectl -n "$OBS_NS" apply -f "${REPO_ROOT}/ops/obs/alerts/atlas-alert-rules.yaml"
    else
      echo "PrometheusRule CRD not present; continuing without rule install"
    fi
    if [ "$PROFILE" = "cluster" ] && ! ops_kubectl api-resources | grep -q "^servicemonitors"; then
      echo "cluster profile requested but ServiceMonitor CRD missing" >&2
      exit 1
    fi
    ;;
  *)
    echo "unknown profile: $PROFILE (expected: local-compose|kind|cluster)" >&2
    exit 1
    ;;
esac

echo "observability pack installed (profile=$PROFILE namespace=$OBS_NS storage_mode=$STORAGE_MODE offline=$OFFLINE_MODE)"
    if [ "$STORAGE_MODE" = "persistent" ]; then
      ops_kubectl -n "$OBS_NS" patch deploy atlas-observability-prometheus --type='json' -p='[{"op":"replace","path":"/spec/template/spec/volumes/1","value":{"name":"data","persistentVolumeClaim":{"claimName":"atlas-observability-prometheus-data"}}}]'
      ops_kubectl -n "$OBS_NS" patch deploy atlas-observability-grafana --type='json' -p='[{"op":"add","path":"/spec/template/spec/volumes/-","value":{"name":"grafana-data","persistentVolumeClaim":{"claimName":"atlas-observability-grafana-data"}}},{"op":"add","path":"/spec/template/spec/containers/0/volumeMounts/-","value":{"name":"grafana-data","mountPath":"/var/lib/grafana"}}]'
    fi
