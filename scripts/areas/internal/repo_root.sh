#!/usr/bin/env sh
# owner: platform
# purpose: print repository root path via package canonical helper.
# stability: internal
set -eu

SCRIPT_DIR="$(CDPATH='' cd -- "$(dirname -- "$0")" && pwd)"
ROOT="$(CDPATH='' cd -- "$SCRIPT_DIR/../../.." && pwd)"
PYTHONPATH="$ROOT/packages/bijux-atlas-scripts/src${PYTHONPATH:+:$PYTHONPATH}" \
  python3 -m bijux_atlas_scripts.internal.cli_compat repo-root
