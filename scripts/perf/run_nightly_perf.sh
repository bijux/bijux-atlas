#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ART="$ROOT/artifacts/perf"
RES="$ART/results"
mkdir -p "$RES" "$ART/cache"

cleanup() {
  docker compose -f "$ROOT/ops/load/docker-compose.perf.yml" down --remove-orphans || true
}
trap cleanup EXIT

"$ROOT/scripts/perf/prepare_perf_store.sh" "$ART/store"

docker compose -f "$ROOT/ops/load/docker-compose.perf.yml" up -d --build

for i in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:18080/readyz" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

OUT_DIR="$ART/cold-start" "$ROOT/scripts/perf/cold_start_benchmark.sh"

"$ROOT/scripts/perf/run_suite.sh" mixed.json "$RES"
"$ROOT/scripts/perf/run_suite.sh" spike.json "$RES"
"$ROOT/scripts/perf/run_suite.sh" stampede.json "$RES"
"$ROOT/scripts/perf/run_suite.sh" sharded-fanout.json "$RES"
"$ROOT/scripts/perf/run_suite.sh" pod-churn.json "$RES"
"$ROOT/scripts/perf/run_suite.sh" response-size-abuse.json "$RES"

# emulate store outage by forcing cached-only mode
ATLAS_CACHED_ONLY_MODE=true docker compose -f "$ROOT/ops/load/docker-compose.perf.yml" up -d --force-recreate atlas-server
for i in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:18080/readyz" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
"$ROOT/scripts/perf/run_suite.sh" store-outage-mid-spike.json "$RES"

# Shorter soak by default for local/nightly cost control; override DURATION=30m
docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats_soak_start.json" || true
DURATION="${SOAK_DURATION:-30m}" "$ROOT/scripts/perf/run_suite.sh" large-dataset-simulation.json "$RES"
docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats_soak_end.json" || true

docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats.json" || true

"$ROOT/scripts/perf/generate_report.py"
"$ROOT/scripts/perf/check_regression.py"
