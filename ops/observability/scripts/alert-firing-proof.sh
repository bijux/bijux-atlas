#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: validate alert contract/rules and prove key alerts are present.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
alerts="$ROOT/ops/observability/alerts/atlas-alert-rules.yaml"
"$ROOT/ops/observability/scripts/alerts-validation.sh"
for a in BijuxAtlasHigh5xxRate BijuxAtlasP95LatencyRegression AtlasOverloadSustained; do
  rg -n "alert:\s*$a" "$alerts" >/dev/null
done
echo "alert firing proof drill passed"
