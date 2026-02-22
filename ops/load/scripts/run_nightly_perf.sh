#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
ART="$ROOT/artifacts/perf"
RES="$ART/results"
mkdir -p "$RES" "$ART/cache"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"

cleanup() {
  docker compose -f "$ROOT/ops/load/compose/docker-compose.perf.yml" down --remove-orphans || true
}
trap cleanup EXIT

"$ROOT/ops/load/scripts/prepare_perf_store.sh" "$ART/store"

docker compose -f "$ROOT/ops/load/compose/docker-compose.perf.yml" up -d --build

for i in $(seq 1 60); do
  if curl -fsS "$ATLAS_BASE_URL/readyz" >/dev/null 2>&1; then
    break
  fi
  sleep 1
done

if command -v kubectl >/dev/null 2>&1; then
  kubectl -n "${ATLAS_E2E_NAMESPACE:-atlas-e2e}" top pods > "$ART/kubectl_top_pods_start.txt" 2>/dev/null || true
fi
OUT_DIR="$ART/cold-start" "$ROOT/ops/load/scripts/cold_start_benchmark.sh"

"$ROOT/ops/load/scripts/check_prereqs.sh"
./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/load/contracts/validate_suite_manifest.py
docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats_soak_start.json" || true
"$ROOT/ops/load/scripts/run_suites_from_manifest.py" --profile nightly --out "$RES"
docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats_soak_end.json" || true

docker stats --no-stream --format '{{json .}}' > "$ART/docker_stats.json" || true
curl -fsS "$ATLAS_BASE_URL/metrics" > "$ART/metrics.prom" 2>/dev/null || true
if command -v kubectl >/dev/null 2>&1; then
  kubectl -n "${ATLAS_E2E_NAMESPACE:-atlas-e2e}" top pods > "$ART/kubectl_top_pods_end.txt" 2>/dev/null || true
fi

./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/load/reports/generate_report.py
"$ROOT/ops/load/scripts/check_regression.py"
./bin/atlasctl run ./packages/atlasctl/src/atlasctl/commands/ops/load/reports/validate_results.py "$RES"
"$ROOT/ops/load/reports/generate.py"
