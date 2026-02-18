#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: canonical deploy entrypoint for atlas chart installation in ops workflows.
# stability: public
# called-by: make ops-deploy
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
exec "$ROOT/ops/e2e/scripts/deploy_atlas.sh" "$@"
