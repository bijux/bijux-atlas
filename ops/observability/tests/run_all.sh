#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/observability/tests/observability-test-lib.sh"

on_fail() {
  out=$(capture_failure_artifacts)
  echo "observability tests failed, artifacts at: $out" >&2
}
trap on_fail ERR

"$ROOT/ops/observability/tests/test_contracts.sh"
"$ROOT/ops/observability/tests/test_profiles.sh"
"$ROOT/ops/observability/tests/test_coverage.sh"
"$ROOT/ops/observability/tests/test_outage_matrix.sh"

echo "observability pack tests passed"
