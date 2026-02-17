#!/usr/bin/env bash
set -euo pipefail

DIR="$(CDPATH= cd -- "$(dirname -- "$0")" && pwd)"

JSON_OUT="${ATLAS_E2E_JSON_OUT:-artifacts/ops/k8s/test-results.json}"
JUNIT_OUT="${ATLAS_E2E_JUNIT_OUT:-artifacts/ops/k8s/test-results.xml}"
RETRIES="${ATLAS_E2E_RETRIES:-1}"

exec "$DIR/harness.py" \
  --manifest "$DIR/manifest.json" \
  --json-out "$JSON_OUT" \
  --junit-out "$JUNIT_OUT" \
  --retries "$RETRIES" \
  "$@"
