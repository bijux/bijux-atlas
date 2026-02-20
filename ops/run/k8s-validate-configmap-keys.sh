#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
ops_env_load
ops_entrypoint_start "ops-k8s-validate-configmap-keys"
ops_version_guard helm kubectl

NS="${1:-${ATLAS_E2E_NAMESPACE:-${ATLAS_NS:-$(ops_layer_ns_k8s)}}}"
SERVICE_NAME="${2:-${ATLAS_E2E_SERVICE_NAME:-$(ops_layer_service_atlas)}}"
CM_NAME="${SERVICE_NAME}-config"
STRICT_MODE="${ATLAS_STRICT_CONFIG_KEYS:-1}"

if [ "$STRICT_MODE" != "1" ]; then
  echo "configmap strict key validation skipped (ATLAS_STRICT_CONFIG_KEYS=${STRICT_MODE})"
  exit 0
fi

tmpl_keys="$(mktemp)"
live_keys="$(mktemp)"
trap 'rm -f "$tmpl_keys" "$live_keys"' EXIT

helm template "$SERVICE_NAME" "$ROOT/ops/k8s/charts/bijux-atlas" -n "$NS" -f "${ATLAS_E2E_VALUES_FILE:-${ATLAS_VALUES_FILE:-$ROOT/ops/k8s/values/local.yaml}}" \
  | awk '
    $0 ~ /^kind: ConfigMap$/ {in_cm=1; next}
    in_cm && $0 ~ /^metadata:/ {next}
    in_cm && $0 ~ /^data:/ {in_data=1; next}
    in_data && $1 ~ /^ATLAS_[A-Z0-9_]+:$/ {gsub(":", "", $1); print $1}
    in_data && $0 !~ /^[[:space:]]/ {in_cm=0; in_data=0}
  ' | sort -u > "$tmpl_keys"

kubectl -n "$NS" get configmap "$CM_NAME" -o jsonpath='{range $k,$v := .data}{$k}{"\n"}{end}' 2>/dev/null | sort -u > "$live_keys"

unknown="$(comm -13 "$tmpl_keys" "$live_keys" || true)"
if [ -n "$unknown" ]; then
  echo "unknown configmap keys detected in ${NS}/${CM_NAME}:" >&2
  echo "$unknown" >&2
  exit 1
fi

echo "configmap key validation passed (${NS}/${CM_NAME})"
