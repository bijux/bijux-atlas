#!/usr/bin/env bash
set -euo pipefail

root="$(cd "$(dirname "$0")/../.." && pwd)"
strict="${STRICT_SCRIPTS_BIN_REMOVAL:-0}"
if [ -d "$root/scripts/bin" ]; then
  if [ "$strict" = "1" ]; then
    echo "forbidden legacy directory exists: scripts/bin" >&2
    exit 1
  fi
  echo "warning: scripts/bin still exists (set STRICT_SCRIPTS_BIN_REMOVAL=1 to enforce removal)" >&2
fi
