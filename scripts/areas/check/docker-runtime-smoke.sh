#!/usr/bin/env bash
# Purpose: compatibility wrapper for canonical docker runtime smoke script.
# Inputs: local built docker image tag via DOCKER_IMAGE.
# Outputs: exits non-zero if canonical smoke checks fail.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
exec "$ROOT/docker/scripts/docker-runtime-smoke.sh" "$@"
