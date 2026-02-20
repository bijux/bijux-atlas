#!/usr/bin/env bash
# owner: platform
# purpose: run a command with timing and structured logs.
# stability: internal
# called-by: scripts/areas/public/report-bundle.sh
# Purpose: execute commands and capture deterministic timing metadata.
# Inputs: command argv and optional SCRIPT_NAME/RUN_ID env vars.
# Outputs: command output and timing file under artifacts/scripts/<name>/<run-id>/.
set -euo pipefail

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
PYTHONPATH="$ROOT/packages/bijux-atlas-scripts/src${PYTHONPATH:+:$PYTHONPATH}" \
  python3 -m bijux_atlas_scripts.internal.cli_compat exec -- "$@"
