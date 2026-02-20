#!/usr/bin/env bash
# owner: operations
# purpose: public wrapper for canonical ops load script prepare_perf_store.sh.
# stability: public
# called-by: make ops-* targets
# Purpose: preserve stable public entrypoint while delegating to ops/load/scripts.
# Inputs: argv passed through unchanged.
# Outputs: same as ops/load/scripts/prepare_perf_store.sh.
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../../.." && pwd)"
exec "$ROOT/ops/load/scripts/prepare_perf_store.sh" "$@"
