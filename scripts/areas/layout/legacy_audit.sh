#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
cd "$ROOT"

echo "legacy audit: scanning tracked files for legacy execution references"
rg -n --hidden --glob '!.git' \
  '(ops-stack-(up|down)-legacy|ops-(check|smoke)-legacy|legacy/[a-z0-9_./-]+|ops/e2e/scripts/(up|down)\.sh)' \
  makefiles docs ops scripts .github configs 2>/dev/null || true
