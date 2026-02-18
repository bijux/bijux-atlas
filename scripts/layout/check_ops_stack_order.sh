#!/usr/bin/env bash
# Purpose: enforce deterministic stack install/uninstall order contract.
# Inputs: ops/e2e/scripts/up.sh and ops/stack/scripts/uninstall.sh.
# Outputs: non-zero on order drift.
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)"
UP="$ROOT/ops/e2e/scripts/up.sh"
DOWN="$ROOT/ops/stack/scripts/uninstall.sh"

assert_order() {
  local file="$1"
  shift
  local prev=0
  local token line
  for token in "$@"; do
    line="$(grep -n "$token" "$file" | head -n1 | cut -d: -f1 || true)"
    if [ -z "$line" ]; then
      echo "stack order contract missing token in $(basename "$file"): $token" >&2
      exit 1
    fi
    if [ "$line" -le "$prev" ]; then
      echo "stack order contract violated in $(basename "$file"): $token" >&2
      exit 1
    fi
    prev="$line"
  done
}

assert_order "$UP" \
  "stack/minio/minio.yaml" \
  "stack/prometheus/prometheus.yaml" \
  "stack/grafana/grafana.yaml"

assert_order "$DOWN" \
  "stack/toxiproxy/toxiproxy.yaml" \
  "stack/redis/redis.yaml" \
  "stack/otel/otel-collector.yaml" \
  "stack/grafana/grafana.yaml" \
  "stack/prometheus/prometheus.yaml" \
  "stack/minio/minio.yaml"

echo "ops stack install/uninstall order contract passed"
