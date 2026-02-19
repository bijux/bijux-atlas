#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-root-lanes"
ops_version_guard python3

MODE="${MODE:-root-local}"
PARALLEL="${PARALLEL:-1}"
RUN_ID="${RUN_ID:-root-lanes-$(date -u +%Y%m%dT%H%M%SZ)}"
SUMMARY_RUN_ID="${SUMMARY_RUN_ID:-$RUN_ID}"
OPEN_SUMMARY="${OPEN_SUMMARY:-0}"

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
)

ROOT_LOCAL_EXTRA_LANES=(
  "internal/lane-ops-smoke"
)

summary_dir_for() {
  local run_id="$1"
  echo "ops/_generated/make/root-local/${run_id}"
}

summary_file_for() {
  local run_id="$1"
  echo "$(summary_dir_for "$run_id")/summary.md"
}

lane_report_path() {
  local lane="$1"
  local run_id="$2"
  echo "ops/_generated/make/${lane}/${run_id}/report.json"
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
  if [ "$MODE" = "root" ]; then
    printf '%s\n' "${ROOT_FAST_LANES[@]}"
    return 0
  fi
  printf '%s\n' "${ALL_LANES[@]}" "${ROOT_LOCAL_EXTRA_LANES[@]}"
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
    echo "- unified: ops/_generated/make/${run_id}/unified.json"
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
  local lane_iso
  lane_iso="$(lane_iso_dir "$lane" "$run_id")"
  local report_root="ops/_generated/make"

  mkdir -p "$(dirname "$lane_log")" "$lane_iso/target" "$lane_iso/cargo-home" "$lane_iso/tmp"

  if ! RUN_ID="$run_id" MAKE_LANE="$lane" TZ="UTC" LANG="C.UTF-8" LC_ALL="C.UTF-8" \
      make -s "$lane" >"$lane_log" 2>&1; then
    lane_status="fail"
  fi

  local lane_end
  lane_end="$(date +%s)"
  local ended_at
  ended_at="$(date -u +%Y-%m-%dT%H:%M:%SZ)"
  local duration="$((lane_end - lane_start))"
  local failure_summary=""
  if [ "$lane_status" != "pass" ]; then
    failure_summary="$(tail -n 20 "$lane_log" 2>/dev/null | tr '\n' ' ' | sed 's/\"/'\''/g')"
  fi
  local lane_artifacts_json
  lane_artifacts_json="$(python3 - <<PY
import json
print(json.dumps([
  "${lane_iso}",
  "${lane_log}",
  "ops/_generated/make/${lane}/${run_id}/report.json"
]))
PY
)"

  LANE_STARTED_AT="$started_at" \
  LANE_ENDED_AT="$ended_at" \
  LANE_ARTIFACT_PATHS_JSON="$lane_artifacts_json" \
  LANE_FAILURE_SUMMARY="$failure_summary" \
  ops_write_lane_report "$lane" "$run_id" "$lane_status" "$duration" "$lane_log" "$report_root" "$started_at" "$ended_at" >/dev/null
  [ "$lane_status" = "pass" ]
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

  mkdir -p "ops/_generated/root-local" "$(summary_dir_for "$run_id")"
  printf '%s\n' "$run_id" > "ops/_generated/root-local/latest-run-id.txt"

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

  RUN_ID="$run_id" ./ops/run/report.sh >/dev/null || true
  python3 ./scripts/layout/check_make_lane_reports.py "$run_id" "${lanes[@]}"
  python3 ./scripts/layout/make_report.py merge --run-id "$run_id" >/dev/null
  write_summary "$run_id" "${lanes[@]}"

  # Isolation guard: lane tmp directories must be unique and scoped under artifacts/isolate.
  python3 ./scripts/layout/check_root_local_lane_isolation.py "$run_id" "${lanes[@]}"

  return "$failed"
}

case "$MODE" in
  summary)
    if [ -z "$SUMMARY_RUN_ID" ] && [ -f "ops/_generated/root-local/latest-run-id.txt" ]; then
      SUMMARY_RUN_ID="$(cat ops/_generated/root-local/latest-run-id.txt)"
    fi
    [ -n "$SUMMARY_RUN_ID" ] || { echo "missing SUMMARY_RUN_ID" >&2; exit 2; }
    print_summary "$SUMMARY_RUN_ID"
    ;;
  open)
    if [ -z "$SUMMARY_RUN_ID" ] && [ -f "ops/_generated/root-local/latest-run-id.txt" ]; then
      SUMMARY_RUN_ID="$(cat ops/_generated/root-local/latest-run-id.txt)"
    fi
    [ -n "$SUMMARY_RUN_ID" ] || { echo "missing SUMMARY_RUN_ID" >&2; exit 2; }
    OPEN_SUMMARY=1
    open_summary "$SUMMARY_RUN_ID"
    ;;
  root|root-local)
    run_lanes "$RUN_ID"
    ;;
  *)
    echo "unknown MODE=${MODE}; expected root|root-local|summary|open" >&2
    exit 2
    ;;
esac
