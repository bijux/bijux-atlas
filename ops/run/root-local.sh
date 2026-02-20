#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_init_run_id
export RUN_ID="$OPS_RUN_ID"
export ARTIFACT_DIR="$OPS_RUN_DIR"
ops_env_load
ops_entrypoint_start "ops-root-local"
ops_version_guard python3

mode="full"
summary_run_id=""
with_ops="${WITH_OPS:-0}"
if [ "${1:-}" = "--fast" ]; then
  mode="fast"
elif [ "${1:-}" = "--with-ops" ]; then
  with_ops="1"
elif [ "${1:-}" = "--summary" ]; then
  mode="summary"
  summary_run_id="${2:-}"
  if [ -z "$summary_run_id" ] && [ -f "ops/_evidence/latest-run-id.txt" ]; then
    summary_run_id="$(cat ops/_evidence/latest-run-id.txt)"
  fi
  [ -n "$summary_run_id" ] || { echo "usage: ops/run/root-local.sh --summary <run_id>" >&2; exit 2; }
fi

lanes=(
  "rust"
  "docs"
  "ops-lint-schemas"
  "layer-contract"
  "stack-smoke"
  "inventories-contracts"
)

lane_cmd() {
  case "$1" in
    rust) echo "make -s root" ;;
    docs) echo "make -s docs" ;;
    ops-lint-schemas) echo "make -s ops-lint ops-contracts-check" ;;
    layer-contract) echo "make -s ops/contract-check" ;;
    stack-smoke)
      if [ "$mode" = "fast" ]; then
        echo "echo 'stack-smoke skipped in fast mode'"
      else
        echo "make -s ops-stack-up PROFILE=kind && make -s ops-stack-smoke && make -s ops-stack-down"
      fi
      ;;
    inventories-contracts) echo "make -s inventory contracts" ;;
    *) echo "unknown lane: $1" >&2; return 2 ;;
  esac
}

print_summary() {
  local run_id="$1"
  echo "root-local summary run_id=$run_id"
  for lane in "${lanes[@]}"; do
    local report="ops/_evidence/${lane}/${run_id}/report.json"
    local status="missing"
    if [ -f "$report" ]; then
      status="$(python3 - <<PY
import json
print(json.loads(open("$report", encoding="utf-8").read()).get("status","unknown"))
PY
)"
    fi
    echo "- ${lane}: ${status} (${report})"
  done
  echo "- unified: ops/_generated_committed/report.unified.json"
}

if [ "$mode" = "summary" ]; then
  print_summary "$summary_run_id"
  exit 0
fi

run_id="${RUN_ID:-root-local-$(date -u +%Y%m%dT%H%M%SZ)}"
mkdir -p ops/_evidence/make/root-local
printf '%s\n' "$run_id" > ops/_evidence/make/root-local/latest-run-id.txt
printf '%s\n' "$run_id" > ops/_evidence/latest-run-id.txt

if [ "$with_ops" = "1" ]; then
  export WITH_OPS=1
fi

pids=()
for lane in "${lanes[@]}"; do
  (
    start="$(date +%s)"
    iso="artifacts/isolate/${lane}/${run_id}"
    report_dir="ops/_evidence/${lane}/${run_id}"
    log="${report_dir}/run.log"
    mkdir -p "$report_dir" "$iso/target" "$iso/cargo-home" "$iso/tmp"

    export ISO_ROOT="$iso"
    export CARGO_TARGET_DIR="$iso/target"
    export CARGO_HOME="$iso/cargo-home"
    export TMPDIR="$iso/tmp"
    export TMP="$iso/tmp"
    export TEMP="$iso/tmp"
    export RUN_ID="${run_id}-${lane}"

    cmd="$(lane_cmd "$lane")"
    status="pass"
    if ! bash -lc "$cmd" >"$log" 2>&1; then
      status="fail"
    fi
    end="$(date +%s)"
    duration="$((end - start))"

    ops_write_lane_report "${lane}" "${run_id}" "${status}" "${duration}" "${log}" "ops/_evidence" >/dev/null

    [ "$status" = "pass" ] || exit 1
  ) &
  pids+=("$!")
done

failed=0
for pid in "${pids[@]}"; do
  if ! wait "$pid"; then
    failed=1
  fi
done

RUN_ID="$run_id" ./ops/run/report.sh >/dev/null || true
print_summary "$run_id"
exit "$failed"
