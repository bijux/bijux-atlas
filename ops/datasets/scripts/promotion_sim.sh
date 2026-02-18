#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: create local promotion workflow skeleton (dev->staging->prod catalogs).
# stability: public
# called-by: make ops-dataset-promotion-sim
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
OUT="${OPS_RUN_DIR:-$ROOT/artifacts/ops/manual}/promotion"
mkdir -p "$OUT"
for env in dev staging prod; do
  cp -f "$ROOT/artifacts/e2e-datasets/catalog.json" "$OUT/catalog.$env.json" 2>/dev/null || echo '{"datasets":[]}' > "$OUT/catalog.$env.json"
done
cp -f "$OUT/catalog.dev.json" "$OUT/catalog.staging.json"
cp -f "$OUT/catalog.staging.json" "$OUT/catalog.prod.json"
echo "promotion simulation written to $OUT"
