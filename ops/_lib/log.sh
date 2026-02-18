#!/usr/bin/env bash
# Purpose: structured JSONL logging for ops entrypoints.
set -euo pipefail

ops_log_json() {
  local level="$1"
  local event="$2"
  local msg="${3:-}"
  python3 - "$level" "$event" "$msg" <<'PY'
import json
import os
import sys
from datetime import datetime, timezone
level, event, msg = sys.argv[1], sys.argv[2], sys.argv[3]
print(json.dumps({
    "ts": datetime.now(timezone.utc).isoformat(),
    "level": level,
    "event": event,
    "msg": msg,
    "run_id": os.environ.get("RUN_ID") or os.environ.get("OPS_RUN_ID"),
    "artifact_dir": os.environ.get("ARTIFACT_DIR") or os.environ.get("OPS_RUN_DIR"),
}, separators=(",", ":")))
PY
}
