#!/usr/bin/env bash
# Purpose: verify image runtime surface for --help/--version on primary binaries.
# Inputs: local built docker image tag via DOCKER_IMAGE.
# Outputs: exits non-zero if runtime surface checks fail.
set -euo pipefail

if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  cat <<'EOF'
Usage: scripts/areas/check/docker-runtime-smoke.sh [image]

Checks --help and --version for bijux-atlas and atlas-server in container image.
EOF
  exit 0
fi

IMAGE="${1:-${DOCKER_IMAGE:-bijux-atlas:local}}"
docker run --rm --entrypoint /app/bijux-atlas "$IMAGE" --help >/dev/null
docker run --rm --entrypoint /app/bijux-atlas "$IMAGE" --version >/dev/null
docker run --rm --entrypoint /app/atlas-server "$IMAGE" --help >/dev/null
docker run --rm --entrypoint /app/atlas-server "$IMAGE" --version >/dev/null

echo "docker runtime smoke passed"
