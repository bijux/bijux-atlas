#!/usr/bin/env bash
set -euo pipefail

./bin/atlasctl check domain docker
./bin/atlasctl run ./ops/run/product_docker_build.sh
./bin/atlasctl docker smoke --image "${DOCKER_IMAGE:-bijux-atlas:local}"
