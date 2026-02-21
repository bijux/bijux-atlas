#!/usr/bin/env bash
# Purpose: apply deterministic repository path migrations.
# Inputs: tracked files in git index/worktree.
# Outputs: updated path references and compatibility layout checks.
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
export PYTHONPATH="$ROOT/packages/atlasctl/src${PYTHONPATH:+:$PYTHONPATH}"
exec python3 -m atlasctl.cli migrate layout
