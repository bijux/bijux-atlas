#!/usr/bin/env bash
set -euo pipefail
DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"
ROOT="$(CDPATH= cd -- "$DIR/../../.." && pwd)"
python3 "$ROOT/scripts/layout/check_tool_versions.py" kind kubectl helm >/dev/null
JSON_OUT="${ATLAS_E2E_JSON_OUT:-ops/_artifacts/k8s/test-results.json}"
JUNIT_OUT="${ATLAS_E2E_JUNIT_OUT:-ops/_artifacts/k8s/test-results.xml}"
RETRIES="${ATLAS_E2E_RETRIES:-1}"
exec "$DIR/harness.py" --manifest "$DIR/manifest.json" --json-out "$JSON_OUT" --junit-out "$JUNIT_OUT" --retries "$RETRIES" "$@"
