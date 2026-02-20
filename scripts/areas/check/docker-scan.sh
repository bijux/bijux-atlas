#!/usr/bin/env bash
# Purpose: compatibility wrapper for canonical docker scan script.
# Inputs: local image tag via DOCKER_IMAGE.
# Outputs: scanner report under artifacts/scripts/docker-scan and non-zero on scanner findings/errors.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
exec "$ROOT/docker/scripts/docker-scan.sh" "$@"
