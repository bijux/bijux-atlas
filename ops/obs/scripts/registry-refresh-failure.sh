#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: assert registry refresh failure behavior and observability signals.
# stability: public
# called-by: make observability-pack-drills
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
"$ROOT/ops/k8s/tests/test_readiness_catalog_refresh.sh"
echo "registry refresh failure drill passed"
