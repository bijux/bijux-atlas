#!/usr/bin/env sh
set -eu

ROOT="$(cd "$(dirname "$0")/../.." && pwd)"
ART="$ROOT/artifacts/perf"
RES="$ART/results"
mkdir -p "$RES" "$ART/cache"

cleanup() {
  docker compose -f "$ROOT/load/docker-compose.perf.yml" down --remove-orphans || true
}
trap cleanup EXIT

"$ROOT/scripts/perf/prepare_perf_store.sh" "$ART/store"

docker compose -f "$ROOT/load/docker-compose.perf.yml" up -d --build

for i in $(seq 1 60); do
  if curl -fsS "http://127.0.0.1:18080/readyz" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

OUT_DIR="$ART/cold-start" "$ROOT/scripts/perf/cold_start_benchmark.sh"

"$ROOT/scripts/perf/run_suite.sh" mixed_80_20.js "$RES"
"$ROOT/scripts/perf/run_suite.sh" warm_steady.js "$RES"
"$ROOT/scripts/perf/run_suite.sh" spike_burst.js "$RES"
"$ROOT/scripts/perf/run_suite.sh" regional_spike_10x_60s.js "$RES"
"$ROOT/scripts/perf/run_suite.sh" cache_stampede.js "$RES"

# emulate store outage by forcing cached-only mode
ATLAS_CACHED_ONLY_MODE=true docker compose -f "$ROOT/load/docker-compose.perf.yml" up -d --force-recreate atlas-server
for i in $(seq 1 30); do
  if curl -fsS "http://127.0.0.1:18080/readyz" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done
"$ROOT/scripts/perf/run_suite.sh" store_outage_mid_spike.js "$RES"

# Shorter soak by default for local/nightly cost control; override DURATION=30m
DURATION="${SOAK_DURATION:-30m}" "$ROOT/scripts/perf/run_suite.sh" soak_30m.js "$RES"

docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats.json" || true

"$ROOT/scripts/perf/generate_report.py"
"$ROOT/scripts/perf/check_regression.py"
