#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ART="$ROOT/artifacts/ops/e2e/k6"
BASE_URL="${BASE_URL:-http://127.0.0.1:18080}"
BASE_URL="${ATLAS_BASE_URL:-$BASE_URL}"
PR_MODE="${PR_MODE:-0}"

mkdir -p "$ART"

profile="full"
if [ "$PR_MODE" = "1" ]; then
  profile="pr"
fi

"$ROOT/ops/load/scripts/validate_suite_manifest.py"
ATLAS_BASE_URL="$BASE_URL" "$ROOT/ops/load/scripts/run_suites_from_manifest.py" --profile "$profile" --out "$ART"

# cold start result
if [ "$PR_MODE" != "1" ]; then
  OUT_DIR="$ART" "$ROOT/ops/load/scripts/cold_start_benchmark.sh" >/dev/null
  if [ -f "$ART/result.json" ]; then
    cp "$ART/result.json" "$ART/cold_start.result.json"
  fi
fi

# scrape metrics once for histogram/metric contract checks
authless_metrics="$(curl -fsS "$BASE_URL/metrics" || true)"
printf "%s\n" "$authless_metrics" > "$ART/metrics.prom"
if [ ! -s "$ART/metrics.prom" ]; then
  echo "runtime metrics snapshot missing: $ART/metrics.prom" >&2
  exit 1
fi

"$ROOT/ops/load/scripts/score_k6.py"
"$ROOT/ops/load/scripts/validate_results.py" "$ART"
"$ROOT/ops/load/reports/generate.py"

echo "e2e perf complete: $ART"
