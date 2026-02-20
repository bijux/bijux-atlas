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
SCRIPT_NAME="${SCRIPT_NAME:-env-dump}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
OUT_DIR="$ROOT/artifacts/scripts/$SCRIPT_NAME/$RUN_ID"
mkdir -p "$OUT_DIR"

{
  echo "pwd=$(pwd)"
  echo "repo_root=$ROOT"
  echo "run_id=$RUN_ID"
  echo "timestamp_utc=$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  env | LC_ALL=C sort
} > "$OUT_DIR/env.txt"

echo "$OUT_DIR/env.txt"
