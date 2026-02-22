#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: prove rolling restart preserves availability.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
"$ROOT/bin/atlasctl" run ./packages/atlasctl/src/atlasctl/commands/ops/k8s/tests/checks/rollout/test_rolling_restart_no_downtime.py
echo "rolling restart no downtime drill passed"
