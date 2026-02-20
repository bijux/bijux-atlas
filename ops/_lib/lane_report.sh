#!/usr/bin/env bash
# Purpose: write lane status JSON payloads compatible with unified report schema.
set -euo pipefail

ops_write_lane_report() {
  local lane="$1"
  local run_id="$2"
  local status="$3"
  local duration_seconds="$4"
  local log_path="$5"
  local out_root="${6:-ops/_evidence}"
  local started_at="${7:-${LANE_STARTED_AT:-}}"
  local ended_at="${8:-${LANE_ENDED_AT:-}}"
  local artifact_paths_json="${LANE_ARTIFACT_PATHS_JSON:-[]}"
  local failure_summary="${LANE_FAILURE_SUMMARY:-}"
  local budget_status_json="${LANE_BUDGET_STATUS_JSON:-null}"
  local repro_command="${LANE_REPRO_COMMAND:-}"

  local lane_dir="${out_root}/${lane}/${run_id}"
  local out_file="${lane_dir}/report.json"
  mkdir -p "$lane_dir"

  python3 - <<PY
import json
from pathlib import Path
payload = {
  "lane": "${lane}",
  "run_id": "${run_id}",
  "status": "${status}",
  "started_at": "${started_at}",
  "ended_at": "${ended_at}",
  "duration_seconds": float("${duration_seconds}"),
  "log": "${log_path}",
  "artifact_paths": json.loads("""${artifact_paths_json}"""),
  "failure_summary": """${failure_summary}""",
  "budget_status": json.loads("""${budget_status_json}"""),
  "repro_command": """${repro_command}""",
}
out = Path("${out_file}")
out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\\n", encoding="utf-8")
print(out.as_posix())
PY
}
