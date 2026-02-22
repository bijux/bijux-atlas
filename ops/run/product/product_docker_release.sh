#!/usr/bin/env bash
set -euo pipefail

if [ "${CI:-0}" != "1" ]; then
  echo "docker-release is CI-only"
  exit 2
fi

./bin/atlasctl run ./ops/run/product_docker_check.sh
./bin/atlasctl run ./ops/run/product_docker_push.sh
