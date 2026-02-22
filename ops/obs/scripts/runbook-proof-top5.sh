#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: verify top runbook mappings and run representative drills.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
for rb in store-outage.md traffic-spike.md rollback-playbook.md pod-churn.md dataset-corruption.md; do
  test -s "$ROOT/docs/operations/runbooks/$rb"
done
for drill in store-outage-under-load overload-admission-control prom-outage otel-outage; do
  python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/drills/run_drill.py" "$drill"
done
echo "runbook proof top5 drill passed"
