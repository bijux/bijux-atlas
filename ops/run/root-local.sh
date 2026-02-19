#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

mode="full"
summary_run_id=""
if [ "${1:-}" = "--fast" ]; then
  mode="fast"
elif [ "${1:-}" = "--summary" ]; then
  mode="summary"
  summary_run_id="${2:-}"
  if [ -z "$summary_run_id" ] && [ -f "ops/_generated/root-local/latest-run-id.txt" ]; then
    summary_run_id="$(cat ops/_generated/root-local/latest-run-id.txt)"
  fi
  [ -n "$summary_run_id" ] || { echo "usage: ops/run/root-local.sh --summary <run_id>" >&2; exit 2; }
fi

lanes=(
  "rust"
  "docs"
  "ops-lint-schemas"
  "stack-smoke"
  "inventories-contracts"
)

lane_cmd() {
  case "$1" in
    rust) echo "make -s root" ;;
    docs) echo "make -s docs" ;;
    ops-lint-schemas) echo "make -s ops-lint ops-contracts-check" ;;
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
    local report="ops/_generated/${lane}/${run_id}.json"
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
  echo "- unified: ops/_generated/report.unified.json"
}

if [ "$mode" = "summary" ]; then
  print_summary "$summary_run_id"
  exit 0
fi

run_id="${RUN_ID:-root-local-$(date -u +%Y%m%dT%H%M%SZ)}"
mkdir -p ops/_generated/root-local
printf '%s\n' "$run_id" > ops/_generated/root-local/latest-run-id.txt

pids=()
for lane in "${lanes[@]}"; do
  (
    start="$(date +%s)"
    iso="artifacts/isolate/${lane}/${run_id}"
    log="ops/_generated/${lane}/${run_id}.log"
    report="ops/_generated/${lane}/${run_id}.json"
    mkdir -p "$(dirname "$log")" "$iso/target" "$iso/cargo-home" "$iso/tmp" "$(dirname "$report")"

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

    python3 - <<PY
import json
from pathlib import Path
payload = {
  "lane": "${lane}",
  "run_id": "${run_id}",
  "status": "${status}",
  "duration_seconds": ${duration},
  "log": "${log}",
  "isolate_root": "${iso}",
}
out = Path("${report}")
out.write_text(json.dumps(payload, indent=2) + "\\n", encoding="utf-8")
print(out)
PY

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
