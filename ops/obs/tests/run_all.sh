#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"

on_fail() {
  out=$(capture_failure_artifacts)
  echo "observability tests failed, artifacts at: $out" >&2
}
trap on_fail ERR

"$ROOT/ops/obs/tests/test_contracts.sh"
"$ROOT/ops/obs/tests/test_profiles.sh"
"$ROOT/ops/obs/tests/test_coverage.sh"
"$ROOT/ops/obs/tests/test_drills.sh"

echo "observability pack tests passed"
