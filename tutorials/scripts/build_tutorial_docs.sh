#!/usr/bin/env bash
set -euo pipefail

if command -v mkdocs >/dev/null 2>&1; then
  mkdocs build --strict
else
  echo "mkdocs not installed; skipping build"
fi
