#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
source "$ROOT/ops/_lib/common.sh"
BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:$(ops_layer_port_atlas)}"
NS="${ATLAS_E2E_NAMESPACE:-$(ops_layer_ns_e2e)}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-$(ops_layer_contract_get release_metadata.defaults.release_name)}"
LOCAL_PORT="${ATLAS_E2E_LOCAL_PORT:-$(ops_layer_port_atlas)}"
CURL_CONNECT_TIMEOUT="${ATLAS_SMOKE_CONNECT_TIMEOUT_SECS:-2}"
CURL_MAX_TIME="${ATLAS_SMOKE_MAX_TIME_SECS:-5}"
SMOKE_HEALTH_RETRIES="${ATLAS_SMOKE_HEALTH_RETRIES:-20}"
CURL="curl --connect-timeout ${CURL_CONNECT_TIMEOUT} --max-time ${CURL_MAX_TIME} -fsS"
SMOKE_DIR="$(ops_artifact_dir smoke)"
PF_LOG="$SMOKE_DIR/port-forward.log"
OUT="$SMOKE_DIR/requests.log"
: > "$OUT"
LOCKFILE="$ROOT/ops/e2e/smoke/queries.lock"
GOLDEN_STATUS="$ROOT/ops/e2e/smoke/goldens/status_codes.json"
RESP_JSON="$SMOKE_DIR/responses.jsonl"
RUN_ID="${RUN_ID:-${OPS_RUN_ID:-local}}"
QUERY_RETRIES="${ATLAS_SMOKE_QUERY_RETRIES:-3}"
: > "$RESP_JSON"

if ! $CURL "$BASE_URL/healthz" >/dev/null 2>&1; then
  POD="$(ops_kubectl -n "$NS" get pods -l app.kubernetes.io/instance="$RELEASE" --field-selector=status.phase=Running -o name | tail -n1 | cut -d/ -f2)"
  ops_kubectl -n "$NS" port-forward "pod/$POD" "$LOCAL_PORT:$(ops_layer_port_atlas)" >"$PF_LOG" 2>&1 &
  PF_PID=$!
  trap 'kill "$PF_PID" >/dev/null 2>&1 || true' EXIT INT TERM
  trap 'ops_kubectl_dump_bundle "$NS" "$(ops_artifact_dir failure-bundle)"' ERR
  BASE_URL="http://127.0.0.1:$LOCAL_PORT"
  for _ in $(seq 1 "$SMOKE_HEALTH_RETRIES"); do
    if $CURL "$BASE_URL/healthz" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done
  if ! $CURL "$BASE_URL/healthz" >/dev/null 2>&1; then
    echo "smoke failed: service not healthy after port-forward retries (base_url=$BASE_URL)" >&2
    ops_kubectl_dump_bundle "$NS" "$(ops_artifact_dir failure-bundle)"
    exit 1
  fi
fi

datasets_body="$($CURL "$BASE_URL/v1/datasets" || true)"
HAS_DATASETS=0
if echo "$datasets_body" | grep -q '"dataset"'; then
  HAS_DATASETS=1
fi

while IFS= read -r q; do
  [ -n "$q" ] || continue
  case "$q" in
    /v1/genes*|/v1/transcripts*|/v1/diff/*|/v1/sequence/*|/v1/releases/*|/v1/datasets/*)
      [ "$HAS_DATASETS" = "1" ] || continue
      ;;
  esac
  status=""
  body=""
  for attempt in $(seq 1 "$QUERY_RETRIES"); do
    status="$(curl --connect-timeout "$CURL_CONNECT_TIMEOUT" --max-time "$CURL_MAX_TIME" -sS -o "$SMOKE_DIR/body.tmp" -w '%{http_code}' "$BASE_URL$q" || true)"
    body="$(cat "$SMOKE_DIR/body.tmp" 2>/dev/null || true)"
    rm -f "$SMOKE_DIR/body.tmp"
    if [ "$status" != "000" ] && [ -n "$status" ]; then
      break
    fi
    sleep 1
  done
  expected="$(python3 -c 'import json,sys; p=sys.argv[1]; f=sys.argv[2]; d=json.load(open(f)); print(d.get(p,\"\"))' "$q" "$GOLDEN_STATUS" 2>/dev/null || true)"
  if [ -n "$expected" ] && [ "$status" != "$expected" ]; then
    echo "status mismatch for $q expected=$expected got=$status" >&2
    exit 1
  fi
  case "$q" in
    /metrics) echo "$body" | grep -q '^bijux_' ;;
    /healthz|/readyz) [ -n "$body" ] ;;
    *) [ -n "$body" ] ;;
  esac
  printf '{"run_id":"%s","path":"%s","status":%s}\n' "$RUN_ID" "$q" "${status:-0}" >> "$RESP_JSON"
  echo "ok $q" | tee -a "$OUT"
done < "$LOCKFILE"
