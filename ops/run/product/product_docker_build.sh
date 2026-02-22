#!/usr/bin/env bash
set -euo pipefail

IMAGE_TAG="${DOCKER_IMAGE:-bijux-atlas:local}"
IMAGE_VERSION="${IMAGE_VERSION:-$(git rev-parse --short=12 HEAD)}"
VCS_REF="${VCS_REF:-$(git rev-parse HEAD)}"
BUILD_DATE="${BUILD_DATE:-$(date -u +%Y-%m-%dT%H:%M:%SZ)}"
RUST_VERSION="${RUST_VERSION:-1.84.1}"

docker build --pull=false -t "${IMAGE_TAG}" -f docker/images/runtime/Dockerfile \
  --build-arg RUST_VERSION="${RUST_VERSION}" \
  --build-arg IMAGE_VERSION="${IMAGE_VERSION}" \
  --build-arg VCS_REF="${VCS_REF}" \
  --build-arg BUILD_DATE="${BUILD_DATE}" \
  --build-arg IMAGE_PROVENANCE="${IMAGE_PROVENANCE:-${IMAGE_TAG}}" \
  .
