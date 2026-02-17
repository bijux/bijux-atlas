#!/usr/bin/env bash
# Purpose: public compatibility wrapper for perf tooling script.
# Inputs: command-line args and env vars.
# Outputs: delegates execution to canonical ops/load/scripts implementation.
# Owner: performance
# Stability: public
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
exec "$ROOT/ops/load/scripts/$(basename "$0")" "$@"
