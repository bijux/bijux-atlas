#!/usr/bin/env bash
# Purpose: compatibility wrapper for canonical docker scan script.
# Inputs: local image tag via DOCKER_IMAGE.
# Outputs: scanner report under artifacts/scripts/docker-scan and non-zero on scanner findings/errors.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
export PYTHONPATH="$ROOT/packages/bijux-atlas-scripts/src${PYTHONPATH:+:$PYTHONPATH}"
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  exec python3 -m bijux_atlas_scripts.cli docker scan --help
fi
exec python3 -m bijux_atlas_scripts.cli docker scan --image "${1:-bijux-atlas:local}"
