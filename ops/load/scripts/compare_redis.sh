#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
OUT="$ROOT/artifacts/perf/redis-compare"
mkdir -p "$OUT/no-redis" "$OUT/with-redis" "$ROOT/artifacts/perf/cache"

"$ROOT/ops/load/scripts/prepare_perf_store.sh" "$ROOT/artifacts/perf/store"

run_stack() {
  compose_file="$1"
  port="$2"
  out_dir="$3"
  docker compose -f "$compose_file" down --remove-orphans || true
  docker compose -f "$compose_file" up -d --build
  for i in $(seq 1 60); do
    if curl -fsS "http://127.0.0.1:${port}/readyz" >/dev/null 2>&1; then
      break
    fi
    sleep 1
  done
  ATLAS_BASE_URL="http://127.0.0.1:${port}" RATE=2000 DURATION=90s "$ROOT/ops/load/scripts/run_suite.sh" mixed.json "$out_dir"
  docker compose -f "$compose_file" down --remove-orphans || true
}

run_stack "$ROOT/ops/load/compose/docker-compose.perf.yml" 18080 "$OUT/no-redis"
run_stack "$ROOT/ops/load/compose/docker-compose.perf.redis.yml" 18081 "$OUT/with-redis"

python3 - <<PY
import json
from pathlib import Path
root = Path("$OUT")

def read(path):
    data = json.loads(path.read_text())
    v = data.get("metrics", {}).get("http_req_duration", {}).get("values", {})
    f = data.get("metrics", {}).get("http_req_failed", {}).get("values", {})
    return {
        "p50": float(v.get("p(50)", 0.0)),
        "p95": float(v.get("p(95)", 0.0)),
        "p99": float(v.get("p(99)", 0.0)),
        "fail": float(f.get("rate", 0.0)),
    }

n = read(root / "no-redis/mixed.summary.json")
r = read(root / "with-redis/mixed.summary.json")

lines = [
    "# Redis Perf Comparison (10x mixed load)",
    "",
    "| mode | p50 ms | p95 ms | p99 ms | fail rate |",
    "|---|---:|---:|---:|---:|",
    f"| no redis | {n['p50']:.2f} | {n['p95']:.2f} | {n['p99']:.2f} | {n['fail']:.4f} |",
    f"| redis enabled | {r['p50']:.2f} | {r['p95']:.2f} | {r['p99']:.2f} | {r['fail']:.4f} |",
    "",
    f"p95 improvement (ms): {n['p95'] - r['p95']:.2f}",
]
(root / "comparison.md").write_text("\n".join(lines) + "\n")
print(root / "comparison.md")
PY
