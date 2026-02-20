#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: ensure dashboards contain expected fault signature panels after drills.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
file="$ROOT/ops/obs/grafana/atlas-observability-dashboard.json"
for key in "shed rate" "bulkhead saturation" "cache hit ratio" "store p95"; do
  rg -ni "$key" "$file" >/dev/null
done
python3 "$ROOT/packages/atlasctl/src/atlasctl/obs/contracts/check_dashboard_contract.py"
echo "dashboard fault signature drill passed"
