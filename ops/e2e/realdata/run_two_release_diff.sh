#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
REAL_ROOT="${ATLAS_REALDATA_ROOT:-$ROOT/artifacts/real-datasets}"
BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"
export ATLAS_REALDATA_ROOT="$REAL_ROOT"

"$ROOT/scripts/fixtures/fetch-real-datasets.sh"
"$ROOT/ops/e2e/scripts/cleanup_store.sh"

"$ROOT/ops/datasets/scripts/publish_by_name.sh" real110
"$ROOT/ops/datasets/scripts/publish_by_name.sh" real111

DIFF_OUT="${ATLAS_E2E_DIFF_OUT:-$ROOT/artifacts/ops/release-diff/110_to_111}"
STORE_ROOT="${ATLAS_E2E_STORE_ROOT:-$ROOT/artifacts/e2e-store}"
mkdir -p "$DIFF_OUT"
cargo run -q -p bijux-atlas-cli --bin bijux-atlas -- atlas diff build \
  --root "$STORE_ROOT" \
  --from-release 110 \
  --to-release 111 \
  --species homo_sapiens \
  --assembly GRCh38 \
  --out-dir "$DIFF_OUT"
test -f "$DIFF_OUT/diff.json"
test -f "$DIFF_OUT/diff.summary.json"

"$ROOT/ops/run/deploy-atlas.sh"

DIFF_GENES="$BASE_URL/v1/diff/genes?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&limit=50"
DIFF_REGION="$BASE_URL/v1/diff/region?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&region=chrA:1-80&limit=50"

curl -fsS "$DIFF_GENES" | python3 -c 'import json,sys; j=json.load(sys.stdin); assert j.get("diff",{}).get("rows") is not None'
curl -fsS "$DIFF_REGION" | python3 -c 'import json,sys; j=json.load(sys.stdin); assert j.get("diff",{}).get("rows") is not None'

echo "realdata two-release diff scenario passed"
