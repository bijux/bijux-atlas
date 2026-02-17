#!/usr/bin/env bash
# owner: platform
# purpose: run a command with timing and structured logs.
# stability: internal
# called-by: scripts/public/report_bundle.sh
# Purpose: execute commands and capture deterministic timing metadata.
# Inputs: command argv and optional SCRIPT_NAME/RUN_ID env vars.
# Outputs: command output and timing file under artifacts/scripts/<name>/<run-id>/.
set -euo pipefail

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)"
SCRIPT_NAME="${SCRIPT_NAME:-exec}"
RUN_ID="${RUN_ID:-$(date -u +%Y%m%dT%H%M%SZ)}"
OUT_DIR="$ROOT/artifacts/scripts/$SCRIPT_NAME/$RUN_ID"
mkdir -p "$OUT_DIR"

start_epoch="$(date +%s)"
"$@"
end_epoch="$(date +%s)"

cat > "$OUT_DIR/timing.json" <<JSON
{"script":"$SCRIPT_NAME","run_id":"$RUN_ID","started":$start_epoch,"ended":$end_epoch,"duration_sec":$((end_epoch-start_epoch))}
JSON
