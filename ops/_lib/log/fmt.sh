#!/usr/bin/env bash
# Purpose: format ops YAML/JSON deterministically.
set -euo pipefail

ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"

find ops configs/ops -type f -name '*.json' ! -path '*/_generated/*' -print0 | while IFS= read -r -d '' f; do
  tmp="$(mktemp)"
  jq -S . "$f" >"$tmp"
  mv "$tmp" "$f"
done

if command -v yq >/dev/null 2>&1; then
  find ops configs/ops -type f \( -name '*.yaml' -o -name '*.yml' \) ! -path '*/_generated/*' -print0 | while IFS= read -r -d '' f; do
    yq -P -i '.' "$f"
  done
else
  echo "yq not installed; YAML formatting skipped" >&2
fi
