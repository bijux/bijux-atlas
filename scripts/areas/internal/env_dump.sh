#!/usr/bin/env bash
# owner: platform
# purpose: capture deterministic environment snapshot for script runs.
# stability: internal
# called-by: scripts/areas/public/report-bundle.sh
# Purpose: dump stable environment diagnostics for troubleshooting.
# Inputs: optional SCRIPT_NAME/RUN_ID env vars.
# Outputs: env snapshot under artifacts/scripts/<name>/<run-id>/env.txt.
set -euo pipefail

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
PYTHONPATH="$ROOT/packages/bijux-atlas-scripts/src${PYTHONPATH:+:$PYTHONPATH}" \
  python3 -m bijux_atlas_scripts.internal.cli_compat env-dump
