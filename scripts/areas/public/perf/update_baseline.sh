#!/usr/bin/env bash
# owner: operations
# purpose: public wrapper for canonical ops load script update_baseline.sh.
# stability: public
# called-by: make ops-* targets
# Purpose: preserve stable public entrypoint while delegating to ops/load/scripts.
# Inputs: argv passed through unchanged.
# Outputs: same as ops/load/scripts/update_baseline.sh.
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../../.." && pwd)"
exec "$ROOT/ops/load/scripts/update_baseline.sh" "$@"
