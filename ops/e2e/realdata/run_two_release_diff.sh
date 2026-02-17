#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
REAL_ROOT="${ATLAS_REALDATA_ROOT:-$ROOT/artifacts/real-datasets}"
BASE_URL="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:18080}"

"$ROOT/scripts/fixtures/fetch-real-datasets.sh"
"$ROOT/ops/e2e/scripts/cleanup_store.sh"

for rel in 110 111; do
  "$ROOT/ops/e2e/scripts/publish_dataset.sh" \
    --gff3 "$REAL_ROOT/$rel/homo_sapiens/GRCh38/genes.gff3" \
    --fasta "$REAL_ROOT/$rel/homo_sapiens/GRCh38/genome.fa" \
    --fai "$REAL_ROOT/$rel/homo_sapiens/GRCh38/genome.fa.fai" \
    --release "$rel" --species homo_sapiens --assembly GRCh38
done

"$ROOT/ops/e2e/scripts/deploy_atlas.sh"

DIFF_GENES="$BASE_URL/v1/diff/genes?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&limit=50"
DIFF_REGION="$BASE_URL/v1/diff/region?from_release=110&to_release=111&species=homo_sapiens&assembly=GRCh38&region=chrA:1-80&limit=50"

curl -fsS "$DIFF_GENES" | python3 -c 'import json,sys; j=json.load(sys.stdin); assert j.get("diff",{}).get("rows") is not None'
curl -fsS "$DIFF_REGION" | python3 -c 'import json,sys; j=json.load(sys.stdin); assert j.get("diff",{}).get("rows") is not None'

echo "realdata two-release diff scenario passed"
