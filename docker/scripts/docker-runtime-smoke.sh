#!/usr/bin/env bash
# Purpose: verify container runtime surface and HTTP endpoints.
set -euo pipefail

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  cat <<'USAGE'
Usage: docker/scripts/docker-runtime-smoke.sh [image]

Checks:
- /app/bijux-atlas --help/--version
- /app/atlas-server --help/--version
- HTTP endpoint reachability: /healthz, /metrics, /v1/version (non-404)
USAGE
  exit 0
fi

IMAGE="${1:-${DOCKER_IMAGE:-bijux-atlas:local}}"

# Binary surface checks
docker run --rm --entrypoint /app/bijux-atlas "$IMAGE" --help >/dev/null
docker run --rm --entrypoint /app/bijux-atlas "$IMAGE" --version >/dev/null
docker run --rm --entrypoint /app/atlas-server "$IMAGE" --help >/dev/null
docker run --rm --entrypoint /app/atlas-server "$IMAGE" --version >/dev/null

# Endpoint checks against a running container
cid="$(docker run -d -p 18080:8080 \
  -e ATLAS_BIND=0.0.0.0:8080 \
  -e ATLAS_STARTUP_WARMUP=0 \
  "$IMAGE")"
cleanup() {
  docker rm -f "$cid" >/dev/null 2>&1 || true
}
trap cleanup EXIT

for _ in 1 2 3 4 5 6 7 8 9 10; do
  code="$(curl -s -o /dev/null -w '%{http_code}' http://127.0.0.1:18080/healthz || true)"
  if [ "$code" != "000" ]; then
    break
  fi
  sleep 1
done

for ep in /healthz /metrics /v1/version; do
  code="$(curl -s -o /dev/null -w '%{http_code}' "http://127.0.0.1:18080$ep" || true)"
  if [ "$code" = "000" ] || [ "$code" = "404" ]; then
    echo "docker runtime smoke failed for $ep (http=$code)" >&2
    exit 1
  fi
done

echo "docker runtime smoke passed"
