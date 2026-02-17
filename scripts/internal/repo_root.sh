#!/usr/bin/env sh
# owner: platform
# purpose: print repository root path from any script location.
# stability: internal
# called-by: scripts/public/report_bundle.sh
# Purpose: resolve repository root deterministically without implicit cwd.
# Inputs: script location only.
# Outputs: absolute repo root path on stdout.
set -eu

SCRIPT_DIR="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
REPO_ROOT="$(CDPATH='' cd -- "$SCRIPT_DIR/../.." && pwd)"
printf '%s\n' "$REPO_ROOT"
