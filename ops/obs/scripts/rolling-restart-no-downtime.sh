#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: prove rolling restart preserves availability.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
"$ROOT/ops/k8s/tests/test_rolling_restart_no_downtime.sh"
echo "rolling restart no downtime drill passed"
