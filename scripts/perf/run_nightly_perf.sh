#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ART="$ROOT/artifacts/perf"
RES="$ART/results"
mkdir -p "$RES" "$ART/cache"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

cleanup() {
  docker compose -f "$ROOT/ops/load/compose/docker-compose.perf.yml" down --remove-orphans || true
}
trap cleanup EXIT

"$ROOT/scripts/perf/prepare_perf_store.sh" "$ART/store"

docker compose -f "$ROOT/ops/load/compose/docker-compose.perf.yml" up -d --build

for i in $(seq 1 60); do
  if curl -fsS "$ATLAS_BASE_URL/readyz" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

OUT_DIR="$ART/cold-start" "$ROOT/scripts/perf/cold_start_benchmark.sh"

"$ROOT/ops/load/scripts/check_prereqs.sh"
"$ROOT/scripts/perf/validate_suite_manifest.py"
docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats_soak_start.json" || true
"$ROOT/scripts/perf/run_suites_from_manifest.py" --profile nightly --out "$RES"
docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats_soak_end.json" || true

docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats.json" || true

"$ROOT/scripts/perf/generate_report.py"
"$ROOT/scripts/perf/check_regression.py"
"$ROOT/scripts/perf/validate_results.py" "$RES"
"$ROOT/ops/load/reports/generate.py"
