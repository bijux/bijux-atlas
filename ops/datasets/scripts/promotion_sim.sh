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
python3 - "$OUT" <<'PY'
import json
import os
from datetime import datetime, timezone
from pathlib import Path
out = Path(os.sys.argv[1])
dev = json.loads((out / "catalog.dev.json").read_text(encoding="utf-8"))
count = len(dev.get("datasets", [])) if isinstance(dev, dict) else 0
report = {
    "schema_version": 1,
    "run_id": os.environ.get("RUN_ID") or os.environ.get("OPS_RUN_ID") or "manual",
    "timestamp_utc": datetime.now(timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
    "source_catalog": "catalog.dev.json",
    "environments": ["dev", "staging", "prod"],
    "promoted_count": count,
}
(out / "promotion-report.json").write_text(json.dumps(report, indent=2) + "\n", encoding="utf-8")
schema = json.loads((Path("ops/_schemas/datasets/promotion-report.schema.json")).read_text(encoding="utf-8"))
for key in schema.get("required", []):
    if key not in report:
        raise SystemExit(f"promotion report missing required key: {key}")
PY
echo "promotion simulation written to $OUT"
