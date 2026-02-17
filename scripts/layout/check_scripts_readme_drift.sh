#!/usr/bin/env bash
# Purpose: ensure scripts/README.md stays generated and current.
# Inputs: scripts tree and scripts/generate_scripts_readme.py.
# Outputs: exits non-zero on drift.
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
python3 "$ROOT/scripts/generate_scripts_readme.py" >/dev/null
if ! git -C "$ROOT" diff --quiet -- scripts/README.md; then
  echo "scripts/README.md drift detected; run: make scripts-index" >&2
  git -C "$ROOT" --no-pager diff -- scripts/README.md >&2 || true
  exit 1
fi
echo "scripts README drift check passed"
