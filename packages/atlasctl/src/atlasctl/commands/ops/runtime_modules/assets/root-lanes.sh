#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/assets/lib/ops_common.sh"
ops_init_run_id
RUN_ID="${RUN_ID:-root-lanes-$(date -u +%Y%m%dT%H%M%SZ)}"
export OPS_RUN_ID="${OPS_RUN_ID:-$RUN_ID}"
export OPS_RUN_DIR="${OPS_RUN_DIR:-$ROOT/artifacts/ops/$OPS_RUN_ID}"
ops_env_load
ops_entrypoint_start "ops-root-lanes"
ops_version_guard python3

MODE="${MODE:-root-local}"
PARALLEL="${PARALLEL:-1}"
SUMMARY_RUN_ID="${SUMMARY_RUN_ID:-$RUN_ID}"
OPEN_SUMMARY="${OPEN_SUMMARY:-0}"
SOURCE_RUN_ID="${SOURCE_RUN_ID:-}"
FAST="${FAST:-0}"
LANE_TIMEOUT_SOFT_SECS="${LANE_TIMEOUT_SOFT_SECS:-1200}"
LANE_TIMEOUT_HARD_SECS="${LANE_TIMEOUT_HARD_SECS:-1800}"

ALL_LANES=(
  "lane-cargo"
  "lane-docs"
  "lane-ops"
  "lane-scripts"
  "lane-configs-policies"
)

ROOT_FAST_LANES=(
  "lane-cargo"
  "lane-docs"
  "lane-scripts"
  "lane-configs-policies"
  "internal/lane-obs-cheap"
)

ROOT_LOCAL_EXTRA_LANES=(
  "internal/lane-ops-smoke"
  "internal/lane-obs-full"
)

summary_dir_for() {
  local run_id="$1"
  echo "artifacts/evidence/make/root-local/${run_id}"
}

summary_file_for() {
  local run_id="$1"
  echo "$(summary_dir_for "$run_id")/summary.md"
}

lane_report_path() {
  local lane="$1"
  local run_id="$2"
  echo "artifacts/evidence/make/${lane}/${run_id}/report.json"
}

lane_iso_dir() {
  local lane="$1"
  local run_id="$2"
  echo "artifacts/isolate/${lane}/${run_id}"
}

lane_log_path() {
  local lane="$1"
  local run_id="$2"
  echo "$(summary_dir_for "$run_id")/${lane}.log"
}

collect_lanes() {
  local include_ops_smoke=1
  if [ "${NO_OPS:-0}" = "1" ]; then
    include_ops_smoke=0
  fi
  if [ "$MODE" = "root" ]; then
    printf '%s\n' "${ROOT_FAST_LANES[@]}"
    return 0
  fi
  printf '%s\n' "${ALL_LANES[@]}"
  if [ "$FAST" = "1" ]; then
    return 0
  fi
  if [ "$include_ops_smoke" = "1" ]; then
    printf '%s\n' "${ROOT_LOCAL_EXTRA_LANES[@]}"
  fi
}

write_summary() {
  local run_id="$1"
  shift
  local lanes=("$@")
  local summary_dir
  summary_dir="$(summary_dir_for "$run_id")"
  local summary_file
  summary_file="$(summary_file_for "$run_id")"

  mkdir -p "$summary_dir"
  {
    echo "# root-local summary"
    echo
    echo "- run_id: ${run_id}"
    echo "- mode: ${MODE}"
    echo "- parallel: ${PARALLEL}"
    echo "- no_ops: ${NO_OPS:-0}"
    echo
    echo "| lane | status | report | isolate | log |"
    echo "|---|---|---|---|---|"
    for lane in "${lanes[@]}"; do
      local report
      report="$(lane_report_path "$lane" "$run_id")"
      local status="missing"
      if [ -f "$report" ]; then
        status="$(python3 - <<PY
import json
print(json.loads(open("$report", encoding="utf-8").read()).get("status","unknown"))
PY
)"
      fi
      echo "| ${lane} | ${status} | ${report} | $(lane_iso_dir "$lane" "$run_id") | $(lane_log_path "$lane" "$run_id") |"
    done
    echo
    echo "- unified: artifacts/evidence/make/${run_id}/unified.json"
    echo "- unified-ops: ops/_generated.example/report.unified.json"
    echo "- scorecard: ops/_generated.example/scorecard.json"
  } > "$summary_file"

  if [ "${QUIET:-0}" != "1" ]; then
    cat "$summary_file"
  fi
}

print_summary() {
  local run_id="$1"
  local lanes=()
  while IFS= read -r lane; do
    [ -n "$lane" ] || continue
    lanes+=("$lane")
  done <<EOF
$(collect_lanes)
EOF
  write_summary "$run_id" "${lanes[@]}"
}

open_summary() {
  local run_id="$1"
  local summary
  summary="$(summary_file_for "$run_id")"
  [ -f "$summary" ] || print_summary "$run_id" >/dev/null
  summary="$(summary_file_for "$run_id")"
  echo "$summary"
  if [ "$OPEN_SUMMARY" = "1" ]; then
    if command -v open >/dev/null 2>&1; then open "$summary" >/dev/null 2>&1 || true; fi
    if command -v xdg-open >/dev/null 2>&1; then xdg-open "$summary" >/dev/null 2>&1 || true; fi
  fi
}

run_lane() {
  local lane="$1"
  local run_id="$2"
  local lane_start
  lane_start="$(date +%s)"
  local started_at
  started_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  local lane_log
  lane_log="$(lane_log_path "$lane" "$run_id")"
  local lane_status="pass"
  local lane_exit_code=0
  local lane_timeout_state="none"
  local lane_iso
  lane_iso="$(lane_iso_dir "$lane" "$run_id")"
  local report_root="artifacts/evidence/make"

  mkdir -p "$(dirname "$lane_log")" "$lane_iso/target" "$lane_iso/cargo-home" "$lane_iso/tmp"

  RUN_ID="$run_id" \
  MAKE_LANE="$lane" \
  ISO_ROOT="$lane_iso" \
  CARGO_TARGET_DIR="$lane_iso/target" \
  CARGO_HOME="$lane_iso/cargo-home" \
  TMPDIR="$lane_iso/tmp" \
  TMP="$lane_iso/tmp" \
  TEMP="$lane_iso/tmp" \
  TZ="UTC" LANG="C.UTF-8" LC_ALL="C.UTF-8" \
  make -s "$lane" >"$lane_log" 2>&1 &
  local lane_pid="$!"
  local soft_marked=0
  while kill -0 "$lane_pid" 2>/dev/null; do
    local now
    now="$(date +%s)"
    local elapsed="$((now - lane_start))"
    if [ "$soft_marked" = "0" ] && [ "$elapsed" -ge "$LANE_TIMEOUT_SOFT_SECS" ]; then
      soft_marked=1
      lane_timeout_state="soft"
      echo "lane timeout warning: lane=$lane run_id=$run_id elapsed=${elapsed}s soft=${LANE_TIMEOUT_SOFT_SECS}s" >>"$lane_log"
    fi
    if [ "$elapsed" -ge "$LANE_TIMEOUT_HARD_SECS" ]; then
      lane_timeout_state="hard"
      echo "lane timeout hard-stop: lane=$lane run_id=$run_id elapsed=${elapsed}s hard=${LANE_TIMEOUT_HARD_SECS}s" >>"$lane_log"
      kill -TERM "$lane_pid" >/dev/null 2>&1 || true
      sleep 2
      kill -KILL "$lane_pid" >/dev/null 2>&1 || true
      break
    fi
    sleep 1
  done
  if ! wait "$lane_pid"; then
    lane_exit_code="$?"
    lane_status="fail"
  fi
  if [ "$lane_timeout_state" = "hard" ] && [ "$lane_exit_code" = "0" ]; then
    lane_exit_code=124
    lane_status="fail"
  fi

  local lane_end
  lane_end="$(date +%s)"
  local ended_at
  ended_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  local duration="$((lane_end - lane_start))"
  local failure_summary=""
  local budget_status_json='null'
  if budget_output="$(python3 ./packages/atlasctl/src/atlasctl/commands/ops/lint/policy/lane_budget_check.py --lane "$lane" --duration-seconds "$duration" 2>&1)"; then
    budget_status_json="$budget_output"
  else
    budget_status_json="$budget_output"
    lane_status="fail"
    failure_summary="budget failure: $(printf '%s' "$budget_output" | tr '\n' ' ' | sed 's/\"/'\''/g')"
  fi
  if [ "$lane_status" != "pass" ]; then
    if [ -z "$failure_summary" ]; then
      failure_summary="exit=${lane_exit_code} timeout=${lane_timeout_state}; $(tail -n 20 "$lane_log" 2>/dev/null | tr '\n' ' ' | sed 's/\"/'\''/g')"
    fi
  fi
  local lane_artifacts_json
  lane_artifacts_json="$(python3 - <<PY
import json
print(json.dumps([
  "${lane_iso}",
  "${lane_log}",
  "artifacts/evidence/make/${lane}/${run_id}/report.json"
]))
PY
)"

  LANE_STARTED_AT="$started_at" \
  LANE_ENDED_AT="$ended_at" \
  LANE_ARTIFACT_PATHS_JSON="$lane_artifacts_json" \
  LANE_FAILURE_SUMMARY="$failure_summary" \
  LANE_BUDGET_STATUS_JSON="$budget_status_json" \
  LANE_TIMEOUT_STATE="$lane_timeout_state" \
  LANE_EXIT_CODE="$lane_exit_code" \
  LANE_REPRO_COMMAND="make repro TARGET=${lane} RUN_ID=${run_id}" \
  ops_write_lane_report "$lane" "$run_id" "$lane_status" "$duration" "$lane_log" "$report_root" "$started_at" "$ended_at" >/dev/null
  [ "$lane_status" = "pass" ]
}

collect_failed_lanes_from_run() {
  local source_run_id="$1"
  python3 - <<PY
import json
from pathlib import Path
root = Path("artifacts/evidence/make")
run = "${source_run_id}"
unified = root / run / "unified.json"
lanes = []
if unified.exists():
    data = json.loads(unified.read_text(encoding="utf-8"))
    for lane, report in sorted(data.get("lanes", {}).items()):
        if report.get("status") != "pass":
            lanes.append(lane)
else:
    for path in root.glob(f"*/{run}/report.json"):
        lane = path.parent.parent.name
        rep = json.loads(path.read_text(encoding="utf-8"))
        if rep.get("status") != "pass":
            lanes.append(lane)
for lane in lanes:
    print(lane)
PY
}

run_lanes() {
  local run_id="$1"
  local lanes=()
  while IFS= read -r lane; do
    [ -n "$lane" ] || continue
    lanes+=("$lane")
  done <<EOF
$(collect_lanes)
EOF

  mkdir -p "artifacts/evidence/make/root-local" "artifacts/evidence/root-local" "$(summary_dir_for "$run_id")"
  printf '%s\n' "$run_id" > "artifacts/evidence/root-local/latest-run-id.txt"
  printf '%s\n' "$run_id" > "artifacts/evidence/make/root-local/latest-run-id.txt"
  printf '%s\n' "$run_id" > "artifacts/runs/latest-run-id.txt"

  local failed=0
  if [ "$PARALLEL" = "0" ]; then
    for lane in "${lanes[@]}"; do
      if ! run_lane "$lane" "$run_id"; then
        failed=1
      fi
    done
  else
    local pids=()
    local lane_names=()
    for lane in "${lanes[@]}"; do
      ( run_lane "$lane" "$run_id" ) &
      pids+=("$!")
      lane_names+=("$lane")
    done

    local idx=0
    for pid in "${pids[@]}"; do
      if ! wait "$pid"; then
        failed=1
      fi
      idx=$((idx + 1))
    done
  fi

  ./bin/atlasctl report unified --run-id "$run_id" --out ops/_generated.example/report.unified.json >/dev/null || true
  python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_make_lane_reports.py "$run_id" "${lanes[@]}"
  python3 ./packages/atlasctl/src/atlasctl/layout_checks/make_report.py merge --run-id "$run_id" >/dev/null
  write_summary "$run_id" "${lanes[@]}"

  # Isolation guard: lane tmp directories must be unique and scoped under artifacts/isolate/.
  python3 ./packages/atlasctl/src/atlasctl/layout_checks/check_root_local_lane_isolation.py "$run_id" "${lanes[@]}"

  return "$failed"
}

case "$MODE" in
  summary)
    if [ -z "$SUMMARY_RUN_ID" ] && [ -f "artifacts/evidence/root-local/latest-run-id.txt" ]; then
      SUMMARY_RUN_ID="$(cat artifacts/evidence/root-local/latest-run-id.txt)"
    fi
    if [ -z "$SUMMARY_RUN_ID" ] && [ -f "artifacts/runs/latest-run-id.txt" ]; then
      SUMMARY_RUN_ID="$(cat artifacts/runs/latest-run-id.txt)"
    fi
    [ -n "$SUMMARY_RUN_ID" ] || { echo "missing SUMMARY_RUN_ID" >&2; exit 2; }
    print_summary "$SUMMARY_RUN_ID"
    ;;
  open)
    if [ -z "$SUMMARY_RUN_ID" ] && [ -f "artifacts/evidence/root-local/latest-run-id.txt" ]; then
      SUMMARY_RUN_ID="$(cat artifacts/evidence/root-local/latest-run-id.txt)"
    fi
    if [ -z "$SUMMARY_RUN_ID" ] && [ -f "artifacts/runs/latest-run-id.txt" ]; then
      SUMMARY_RUN_ID="$(cat artifacts/runs/latest-run-id.txt)"
    fi
    [ -n "$SUMMARY_RUN_ID" ] || { echo "missing SUMMARY_RUN_ID" >&2; exit 2; }
    OPEN_SUMMARY=1
    open_summary "$SUMMARY_RUN_ID"
    ;;
  root|root-local)
    run_lanes "$RUN_ID"
    ;;
  rerun-failed)
    [ -n "$SOURCE_RUN_ID" ] || { echo "missing SOURCE_RUN_ID" >&2; exit 2; }
    if [ -z "${RUN_ID:-}" ] || [ "$RUN_ID" = "root-lanes-$(date -u +%Y%m%dT%H%M%SZ)" ]; then
      RUN_ID="${SOURCE_RUN_ID}-rerun-$(date -u +%Y%m%dT%H%M%SZ)"
    fi
    failed_lanes=()
    while IFS= read -r lane; do
      [ -n "$lane" ] || continue
      failed_lanes+=("$lane")
    done <<EOF
$(collect_failed_lanes_from_run "$SOURCE_RUN_ID")
EOF
    if [ "${#failed_lanes[@]}" -eq 0 ]; then
      echo "no failed lanes for run_id=${SOURCE_RUN_ID}"
      exit 0
    fi
    collect_lanes() {
      printf '%s\n' "${failed_lanes[@]}"
    }
    run_lanes "$RUN_ID"
    ;;
  *)
    echo "unknown MODE=${MODE}; expected root|root-local|summary|open|rerun-failed" >&2
    exit 2
    ;;
esac
