#!/usr/bin/env bash
set -euo pipefail

if [ "${CI:-0}" != "1" ]; then
  echo "docker-push is CI-only"
  exit 2
fi

IMAGE_TAG="${DOCKER_IMAGE:?DOCKER_IMAGE is required for docker-push}"
docker push "${IMAGE_TAG}"
