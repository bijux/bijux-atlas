#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../../.." && pwd)"
source "$ROOT/ops/_lib/k8s/k8s-test-common.sh"

need kubectl
need helm
need curl

setup_test_traps
ops_kubectl_retry apply -f "$ROOT/ops/stack/redis/redis.yaml"
wait_kubectl_condition deployment redis Available 180s

install_chart \
  --set server.enableDebugDatasets=true \
  --set rateLimits.perIp.capacity=1 \
  --set rateLimits.perIp.refillPerSec=0.1 \
  --set-json extraEnv='[{"name":"ATLAS_ENABLE_REDIS_RATE_LIMIT","value":"true"},{"name":"ATLAS_ENABLE_REDIS_RESPONSE_CACHE","value":"true"},{"name":"ATLAS_REDIS_URL","value":"redis://redis.atlas-e2e.svc.cluster.local:6379/"},{"name":"ATLAS_REDIS_PREFIX","value":"atlas-e2e"}]'

wait_ready
with_port_forward 18080
wait_for_http "$BASE_URL/healthz" 200 120

url="$BASE_URL/v1/datasets"
first="$(curl -s -o /dev/null -w '%{http_code}' "$url" || true)"
second="$(curl -s -o /dev/null -w '%{http_code}' "$url" || true)"
if [ "$first" != "200" ]; then
  echo "expected first request 200, got $first" >&2
  exit 1
fi
if [ "$second" != "429" ]; then
  echo "expected second request 429 with redis rate limit enabled, got $second" >&2
  exit 1
fi

echo "redis rate-limit check passed"
