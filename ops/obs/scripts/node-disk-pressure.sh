#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: run node disk pressure simulation and verify service health.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
"$ROOT/ops/k8s/tests/checks/perf/test_disk_pressure.sh"
echo "node disk pressure drill passed"
