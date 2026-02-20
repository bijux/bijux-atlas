#!/usr/bin/env bash
set -euo pipefail

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
