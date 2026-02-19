#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

run_id="${RUN_ID:-root-local-$(date -u +%Y%m%dT%H%M%SZ)}"
lanes=(
  "rust"
  "docs"
  "ops-lint-schemas"
  "stack-smoke"
  "contracts-inventory"
)

lane_cmd() {
  case "$1" in
    rust) echo "make -s fmt lint test" ;;
    docs) echo "make -s docs-freeze" ;;
    ops-lint-schemas) echo "make -s ops-lint ops-contracts-check" ;;
    stack-smoke) echo "make -s ops-stack-up PROFILE=kind && make -s ops-stack-smoke && make -s ops-stack-down" ;;
    contracts-inventory) echo "make -s contracts scripts-index" ;;
    *) echo "unknown lane: $1" >&2; return 2 ;;
  esac
}

mkdir -p ops/_generated
pids=()
for lane in "${lanes[@]}"; do
  (
    start="$(date +%s)"
    iso="artifacts/isolate/${lane}/${run_id}"
    log="ops/_generated/${lane}/${run_id}.log"
    report="ops/_generated/${lane}/${run_id}.json"
    mkdir -p "$(dirname "$log")" "$iso" "$(dirname "$report")"

    export ISO_ROOT="$iso"
    export CARGO_TARGET_DIR="$iso/target"
    export CARGO_HOME="$iso/cargo-home"
    export TMPDIR="$iso/tmp"
    export TMP="$iso/tmp"
    export TEMP="$iso/tmp"
    export RUN_ID="${run_id}-${lane}"
    export ARTIFACT_DIR="ops/_generated/${lane}"

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
  "log": "${log}"
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

RUN_ID="$run_id" ./ops/run/report.sh >/dev/null

echo "root-local summary run_id=$run_id"
for lane in "${lanes[@]}"; do
  report="ops/_generated/${lane}/${run_id}.json"
  status="unknown"
  if [ -f "$report" ]; then
    status="$(python3 - <<PY
import json
print(json.loads(open("${report}", encoding="utf-8").read()).get("status","unknown"))
PY
)"
  fi
  echo "- ${lane}: ${status} (ops/_generated/${lane}/${run_id}.json)"
done
echo "- unified: ops/_generated/report.unified.json"

exit "$failed"
