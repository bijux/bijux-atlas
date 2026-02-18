#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: run cpu throttle noisy-neighbor scenario and verify suite completion.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
"$ROOT/ops/load/tests/test_cpu_throttle_noisy_neighbor.sh"
echo "cpu throttle noisy neighbor drill passed"
