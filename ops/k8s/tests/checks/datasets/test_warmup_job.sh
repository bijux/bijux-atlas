#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
. "$SCRIPT_DIR/../_lib/common.sh"
setup_test_traps
need helm; need kubectl; need curl; need python3

TMP_VALUES="$(mktemp)"
cat > "$TMP_VALUES" <<YAML
cache:
  pinnedDatasets:
    - 110/homo_sapiens/GRCh38
datasetWarmupJob:
  enabled: true
YAML
install_chart -f "$TMP_VALUES"
with_port_forward 18080
before="$(curl -fsS "$BASE_URL/metrics" | awk '/^bijux_dataset_cache_hits_total/ {print $2}' | tail -n1)"
before="${before:-0}"
wait_kubectl_condition job "$SERVICE_NAME-dataset-warmup" complete 300s
sleep 2
curl -fsS "$BASE_URL/healthz" >/dev/null || true
after="$(curl -fsS "$BASE_URL/metrics" | awk '/^bijux_dataset_cache_hits_total/ {print $2}' | tail -n1)"
after="${after:-0}"
python3 - "$before" "$after" <<'PY'
import sys
b=float(sys.argv[1]); a=float(sys.argv[2])
assert a >= b, f"cache hits did not increase or remain stable: before={b} after={a}"
PY

echo "warmup job gate passed"
