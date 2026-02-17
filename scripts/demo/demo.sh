#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

PORT="${ATLAS_DEMO_PORT:-18082}"
BASE_URL="http://127.0.0.1:${PORT}"
STORE_ROOT="${ATLAS_DEMO_STORE_ROOT:-artifacts/medium-output}"
CACHE_ROOT="${ATLAS_DEMO_CACHE_ROOT:-artifacts/demo-cache}"
DATASET="${ATLAS_DEMO_DATASET:-110/homo_sapiens/GRCh38}"

echo "[demo] fetching medium fixture (deterministic + checksum pinned)"
if ! ./scripts/fixtures/fetch-medium.sh; then
  if [ ! -d "ops/fixtures/medium/data" ]; then
    echo "[demo] medium fixture download failed and ops/fixtures/medium/data is missing" >&2
    exit 1
  fi
  echo "[demo] fetch failed but local ops/fixtures/medium/data exists; continuing"
fi

echo "[demo] ingesting medium fixture to ${STORE_ROOT}"
./scripts/fixtures/run-medium-ingest.sh

echo "[demo] starting server on ${BASE_URL}"
ATLAS_BIND="127.0.0.1:${PORT}" \
ATLAS_STORE_ROOT="${STORE_ROOT}" \
ATLAS_CACHE_ROOT="${CACHE_ROOT}" \
ATLAS_READINESS_REQUIRES_CATALOG=false \
ATLAS_STARTUP_WARMUP="${DATASET}" \
cargo run -p bijux-atlas-server --bin atlas-server >/tmp/bijux-atlas-demo.log 2>&1 &
SERVER_PID=$!
cleanup() {
  kill "$SERVER_PID" >/dev/null 2>&1 || true
}
trap cleanup EXIT INT TERM

echo "[demo] waiting for readiness"
for _ in $(seq 1 60); do
  if curl -fsS "${BASE_URL}/readyz" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

query() {
  name="$1"
  path="$2"
  echo
  echo "== ${name}"
  echo "${BASE_URL}${path}"
  curl -fsS "${BASE_URL}${path}" | python3 -m json.tool | sed -n '1,40p'
}

query "1) Version" "/v1/version"
query "2) Datasets" "/v1/datasets"
query "3) Release Metadata + QC" "/v1/releases/110/species/homo_sapiens/assemblies/GRCh38"
query "4) Gene Count" "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38"
query "5) Gene List" "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=3"
query "6) Region Query" "/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-100000&limit=3"
query "7) Diff Query" "/v1/diff/genes?from_release=109&to_release=110&species=homo_sapiens&assembly=GRCh38&limit=5"
query "8) Sequence Query" "/v1/sequence/region?release=110&species=homo_sapiens&assembly=GRCh38&region=chr1:1-100"
GENE_ID="$(curl -fsS "${BASE_URL}/v1/genes?release=110&species=homo_sapiens&assembly=GRCh38&limit=1" | python3 -c 'import json,sys; d=json.load(sys.stdin); rows=d.get("response",{}).get("rows",[]); print(rows[0]["gene_id"] if rows else "GENE0001")')"
query "9) Gene Sequence Query" "/v1/genes/${GENE_ID}/sequence?release=110&species=homo_sapiens&assembly=GRCh38"
query "10) Gene Transcripts Query" "/v1/genes/${GENE_ID}/transcripts?release=110&species=homo_sapiens&assembly=GRCh38&limit=5"

echo
echo "[demo] dataset browser: ${BASE_URL}/"
echo "[demo] done. server log: /tmp/bijux-atlas-demo.log"