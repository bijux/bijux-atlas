#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: compatibility entrypoint for observability drill runner.
# stability: public
# called-by: human operator
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/.." && pwd)"
exec "$ROOT/ops/observability/scripts/run_drill.sh" "$@"
