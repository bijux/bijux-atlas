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

queries="/healthz
/v1/version
/v1/datasets
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&gene_id=GENE1
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name=Gene1
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&name_prefix=Gene
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&biotype=protein_coding
/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-1000
/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38
/v1/releases/110/species/homo_sapiens/assemblies/GRCh38
/v1/transcripts/TX1?release=110&species=homo_sapiens&assembly=GRCh38
/v1/genes/GENE1/transcripts?release=110&species=homo_sapiens&assembly=GRCh38
/v1/diff/genes?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&limit=10
/v1/diff/region?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-1000
/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-20
/v1/genes/GENE1/sequence?release=110&species=homo_sapiens&assembly=GRCh38
/metrics
/debug/datasets
"

datasets_body="$($CURL "$BASE_URL/v1/datasets" || true)"
HAS_DATASETS=0
if echo "$datasets_body" | grep -q '"dataset"'; then
  HAS_DATASETS=1
fi

for q in $queries; do
  case "$q" in
    /v1/genes*|/v1/transcripts*|/v1/diff/*|/v1/sequence/*|/v1/releases/*)
      [ "$HAS_DATASETS" = "1" ] || continue
      ;;
  esac
  body="$($CURL "$BASE_URL$q")"
  case "$q" in
    /metrics) echo "$body" | grep -q '^bijux_' ;;
    /healthz|/readyz) [ -n "$body" ] ;;
    *) [ -n "$body" ] ;;
  esac
  echo "ok $q" | tee -a "$OUT"
done
