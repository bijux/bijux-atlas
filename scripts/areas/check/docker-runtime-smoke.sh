#!/usr/bin/env bash
# Purpose: compatibility wrapper for canonical docker runtime smoke script.
# Inputs: local built docker image tag via DOCKER_IMAGE.
# Outputs: exits non-zero if canonical smoke checks fail.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
export PYTHONPATH="$ROOT/packages/bijux-atlas-scripts/src${PYTHONPATH:+:$PYTHONPATH}"
if [ "${1:-}" = "--help" ] || [ "${1:-}" = "-h" ]; then
  exec python3 -m bijux_atlas_scripts.cli docker smoke --help
fi
exec python3 -m bijux_atlas_scripts.cli docker smoke --image "${1:-bijux-atlas:local}"
