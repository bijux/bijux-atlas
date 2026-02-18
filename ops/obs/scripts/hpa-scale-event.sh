#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: prove HPA scale up/down event under synthetic load.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
"$ROOT/ops/k8s/tests/test_hpa.sh"
echo "hpa scale event drill passed"
