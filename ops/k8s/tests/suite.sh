#!/usr/bin/env bash
set -euo pipefail
DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT="$(CDPATH= cd -- "$DIR/../../.." && pwd)"
python3 "$ROOT/packages/atlasctl/src/atlasctl/layout_checks/check_tool_versions.py" kind kubectl helm >/dev/null
JSON_OUT="${ATLAS_E2E_JSON_OUT:-ops/_artifacts/k8s/test-results.json}"
JUNIT_OUT="${ATLAS_E2E_JUNIT_OUT:-ops/_artifacts/k8s/test-results.xml}"
SUMMARY_OUT="${ATLAS_E2E_SUMMARY_OUT:-ops/_artifacts/k8s/test-summary.md}"
DEGRADATION_SCORE_OUT="${ATLAS_E2E_DEGRADATION_SCORE_OUT:-$(dirname "$JSON_OUT")/graceful-degradation-score.json}"
CONFORMANCE_OUT="${ATLAS_E2E_CONFORMANCE_OUT:-$(dirname "$JSON_OUT")/k8s-conformance-report.json}"
RETRIES="${ATLAS_E2E_RETRIES:-1}"
SUITE=""
FAIL_FAST="${ATLAS_E2E_FAIL_FAST:-0}"
INCLUDE_QUARANTINED="${ATLAS_E2E_INCLUDE_QUARANTINED:-0}"
if [ "${1:-}" = "--suite" ]; then
  SUITE="${2:-full}"
  shift 2
fi

group_args=()
suite_id="${SUITE:-adhoc}"
if [ -n "$SUITE" ]; then
  while IFS= read -r grp; do
    [ -n "$grp" ] && group_args+=("--group" "$grp")
  done < <(python3 - <<PY
import json
s=json.load(open("$DIR/suites.json", encoding="utf-8"))
name="$SUITE"
for suite in s.get("suites", []):
    if suite.get("id") == name:
        fail_fast = 1 if suite.get("fail_fast", False) else 0
        print(f"__FAIL_FAST__={fail_fast}")
        for g in sorted(suite.get("groups", [])):
            print(g)
        break
else:
    raise SystemExit(f"unknown suite id: {name}")
PY
)
fi

parsed_groups=()
for arg in "${group_args[@]}"; do
  if [[ "$arg" == __FAIL_FAST__=* ]]; then
    if [ "${arg#__FAIL_FAST__=}" = "1" ]; then
      FAIL_FAST=1
    fi
  else
    parsed_groups+=("$arg")
  fi
done

harness_args=(--manifest "$DIR/manifest.json" --json-out "$JSON_OUT" --junit-out "$JUNIT_OUT" --retries "$RETRIES" --suite-id "$suite_id")
if [ "$FAIL_FAST" = "1" ]; then
  harness_args+=(--fail-fast)
fi
if [ "$INCLUDE_QUARANTINED" = "1" ]; then
  harness_args+=(--include-quarantined)
fi
harness_args+=("${parsed_groups[@]}" "$@")

status=0
"$DIR/harness.py" "${harness_args[@]}" || status=$?
python3 "$DIR/validate_report.py" --report "$JSON_OUT"
python3 "$DIR/render_summary.py" --json "$JSON_OUT" --out "$SUMMARY_OUT"
python3 "$DIR/compute_graceful_degradation_score.py" --json "$JSON_OUT" --out "$DEGRADATION_SCORE_OUT"
python3 "$DIR/build_conformance_report.py" --json "$JSON_OUT" --out "$CONFORMANCE_OUT"
exit "$status"
