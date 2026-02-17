#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
OUT_DIR="${OUT_DIR:-$ROOT/artifacts/perf/cold-start-5pods}"
mkdir -p "$OUT_DIR"

BIN="${ATLAS_SERVER_BIN:-$ROOT/artifacts/target/debug/atlas-server}"
if [[ ! -x "$BIN" ]]; then
  echo "atlas-server binary not found at $BIN" >&2
  exit 1
fi

STORE_ROOT="${ATLAS_STORE_ROOT:-$ROOT/artifacts/server-store}"
BASE_PORT="${ATLAS_BASE_PORT:-18080}"
PODS=5

declare -a pids
trap 'for p in "${pids[@]:-}"; do kill "$p" 2>/dev/null || true; done' EXIT

results_json="$OUT_DIR/result.json"
: > "$results_json"

echo "[" >> "$results_json"
for i in $(seq 1 "$PODS"); do
  port=$((BASE_PORT + i - 1))
  cache_root="$ROOT/artifacts/server-cache-pod-$i"
  rm -rf "$cache_root"
  mkdir -p "$cache_root"

  (
    export ATLAS_BIND="127.0.0.1:${port}"
    export ATLAS_STORE_ROOT="$STORE_ROOT"
    export ATLAS_CACHE_ROOT="$cache_root"
    export ATLAS_STARTUP_WARMUP_JITTER_MAX_MS="0"
    "$BIN" >"$OUT_DIR/pod-${i}.log" 2>&1
  ) &
  pids+=("$!")

  start_ms=$(date +%s%3N)
  for _ in $(seq 1 200); do
    code=$(curl -s -o /dev/null -w "%{http_code}" "http://127.0.0.1:${port}/readyz" || true)
    if [[ "$code" == "200" ]]; then
      break
    fi
    sleep 0.1
  done
  end_ms=$(date +%s%3N)
  elapsed=$((end_ms - start_ms))
  printf '  {"pod":%s,"port":%s,"cold_start_ms":%s}%s\n' "$i" "$port" "$elapsed" "$( [[ $i -lt $PODS ]] && echo , )" >> "$results_json"
done

echo "]" >> "$results_json"

echo "wrote $results_json"