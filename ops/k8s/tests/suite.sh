#!/usr/bin/env bash
set -euo pipefail
DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT="$(CDPATH= cd -- "$DIR/../../.." && pwd)"
python3 "$ROOT/scripts/layout/check_tool_versions.py" kind kubectl helm >/dev/null
JSON_OUT="${ATLAS_E2E_JSON_OUT:-ops/_artifacts/k8s/test-results.json}"
JUNIT_OUT="${ATLAS_E2E_JUNIT_OUT:-ops/_artifacts/k8s/test-results.xml}"
RETRIES="${ATLAS_E2E_RETRIES:-1}"
SUITE=""
if [ "${1:-}" = "--suite" ]; then
  SUITE="${2:-full}"
  shift 2
fi

group_args=()
if [ -n "$SUITE" ] && [ "$SUITE" != "full" ]; then
  while IFS= read -r grp; do
    [ -n "$grp" ] && group_args+=("--group" "$grp")
  done < <(python3 - <<PY
import json
s=json.load(open("$DIR/suites.json", encoding="utf-8"))
name="$SUITE"
for suite in s.get("suites", []):
    if suite.get("id") == name:
        for g in suite.get("groups", []):
            print(g)
        break
else:
    raise SystemExit(f"unknown suite id: {name}")
PY
)
fi

if [ "${#group_args[@]}" -gt 0 ]; then
  exec "$DIR/harness.py" --manifest "$DIR/manifest.json" --json-out "$JSON_OUT" --junit-out "$JUNIT_OUT" --retries "$RETRIES" "${group_args[@]}" "$@"
fi
exec "$DIR/harness.py" --manifest "$DIR/manifest.json" --json-out "$JSON_OUT" --junit-out "$JUNIT_OUT" --retries "$RETRIES" "$@"
