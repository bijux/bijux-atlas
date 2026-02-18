#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
source "$ROOT/ops/_lib/common.sh"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
SERVICE_NAME="${ATLAS_E2E_SERVICE_NAME:-$RELEASE-bijux-atlas}"
LOCAL_PORT="${ATLAS_E2E_LOCAL_PORT:-18080}"
CURL="curl --connect-timeout 2 --max-time 30 -fsS"
WARM_DIR="$(ops_artifact_dir warm)"
PF_LOG="$WARM_DIR/port-forward.log"

if ! command -v kubectl >/dev/null 2>&1; then
  echo "kubectl is required" >&2
  exit 1
fi

if ops_kubectl -n "$NS" get job "$SERVICE_NAME-dataset-warmup" >/dev/null 2>&1; then
  ops_kubectl_retry -n "$NS" wait --for=condition=complete --timeout=5m job/"$SERVICE_NAME-dataset-warmup"
fi

POD="$(ops_kubectl -n "$NS" get pods -l app.kubernetes.io/instance="$RELEASE" --field-selector=status.phase=Running -o name | tail -n1 | cut -d/ -f2)"
ops_kubectl -n "$NS" port-forward "pod/$POD" "$LOCAL_PORT:8080" >"$PF_LOG" 2>&1 &
PF_PID=$!
trap 'kill "$PF_PID" >/dev/null 2>&1 || true; ops_kubectl_dump_bundle "$NS" "$(ops_artifact_dir failure-bundle)"' ERR
trap 'kill "$PF_PID" >/dev/null 2>&1 || true' EXIT INT TERM

for _ in 1 2 3 4 5 6 7 8 9 10; do
  if $CURL "http://127.0.0.1:$LOCAL_PORT/healthz" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

METRICS_BEFORE="$(mktemp)"
METRICS_AFTER="$(mktemp)"
trap 'kill "$PF_PID" >/dev/null 2>&1 || true; rm -f "$METRICS_BEFORE" "$METRICS_AFTER"' EXIT INT TERM

$CURL "http://127.0.0.1:$LOCAL_PORT/metrics" >"$METRICS_BEFORE" || true
for _ in 1 2 3; do
  if $CURL "http://127.0.0.1:$LOCAL_PORT/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
for _ in 1 2 3; do
  if $CURL "http://127.0.0.1:$LOCAL_PORT/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
$CURL "http://127.0.0.1:$LOCAL_PORT/metrics" >"$METRICS_AFTER" || true
cp "$METRICS_BEFORE" "$WARM_DIR/metrics.before.prom"
cp "$METRICS_AFTER" "$WARM_DIR/metrics.after.prom"

extract_metric() {
  local file="$1"
  grep -E '^bijux_dataset_hits(\{| )|^bijux_dataset_cache_hit_total(\{| )' "$file" | awk '{print $NF}' | head -n1
}

BEFORE="$(extract_metric "$METRICS_BEFORE" || true)"
AFTER="$(extract_metric "$METRICS_AFTER" || true)"
if [ -n "${BEFORE:-}" ] && [ -n "${AFTER:-}" ]; then
  [ "$AFTER" -ge "$BEFORE" ]
fi

echo "warmup verification completed"
