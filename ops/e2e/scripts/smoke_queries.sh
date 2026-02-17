#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
source "$ROOT/ops/_lib/common.sh"
BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
NS="${ATLAS_E2E_NAMESPACE:-atlas-e2e}"
RELEASE="${ATLAS_E2E_RELEASE_NAME:-atlas-e2e}"
LOCAL_PORT="${ATLAS_E2E_LOCAL_PORT:-18080}"
CURL="curl --connect-timeout 2 --max-time 5 -fsS"
SMOKE_DIR="$(ops_artifact_dir smoke)"
PF_LOG="$SMOKE_DIR/port-forward.log"
OUT="$SMOKE_DIR/requests.log"
: > "$OUT"
LOCKFILE="$ROOT/ops/smoke/queries.lock"
GOLDEN_STATUS="$ROOT/ops/smoke/goldens/status_codes.json"
RESP_JSON="$SMOKE_DIR/responses.jsonl"
: > "$RESP_JSON"

if ! $CURL "$BASE_URL/healthz" >/dev/null 2>&1; then
  POD="$(ops_kubectl -n "$NS" get pods -l app.kubernetes.io/instance="$RELEASE" --field-selector=status.phase=Running -o name | tail -n1 | cut -d/ -f2)"
  ops_kubectl -n "$NS" port-forward "pod/$POD" "$LOCAL_PORT:8080" >"$PF_LOG" 2>&1 &
  PF_PID=$!
  trap 'kill "$PF_PID" >/dev/null 2>&1 || true' EXIT INT TERM
  trap 'ops_kubectl_dump_bundle "$NS" "$(ops_artifact_dir failure-bundle)"' ERR
  BASE_URL="http://127.0.0.1:$LOCAL_PORT"
  for _ in 1 2 3 4 5 6 7 8 9 10; do
    if $CURL "$BASE_URL/healthz" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done
fi

datasets_body="$($CURL "$BASE_URL/v1/datasets" || true)"
HAS_DATASETS=0
if echo "$datasets_body" | grep -q '"dataset"'; then
  HAS_DATASETS=1
fi

while IFS= read -r q; do
  [ -n "$q" ] || continue
  case "$q" in
    /v1/genes*|/v1/transcripts*|/v1/diff/*|/v1/sequence/*|/v1/releases/*)
      [ "$HAS_DATASETS" = "1" ] || continue
      ;;
  esac
  status="$(curl --connect-timeout 2 --max-time 5 -sS -o "$SMOKE_DIR/body.tmp" -w '%{http_code}' "$BASE_URL$q" || true)"
  body="$(cat "$SMOKE_DIR/body.tmp" 2>/dev/null || true)"
  rm -f "$SMOKE_DIR/body.tmp"
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
  printf '{"path":"%s","status":%s}\n' "$q" "${status:-0}" >> "$RESP_JSON"
  echo "ok $q" | tee -a "$OUT"
done < "$LOCKFILE"
