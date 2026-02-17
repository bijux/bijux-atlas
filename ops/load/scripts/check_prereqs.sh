#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
ATLAS_BASE_URL="${ATLAS_BASE_URL:-http://127.0.0.1:18080}"
command -v k6 >/dev/null 2>&1 || { echo "k6 required" >&2; exit 1; }
command -v curl >/dev/null 2>&1 || { echo "curl required" >&2; exit 1; }
if ! curl -fsS "$ATLAS_BASE_URL/healthz" >/dev/null 2>&1; then
  echo "atlas endpoint not reachable at $ATLAS_BASE_URL" >&2
  exit 1
fi
echo "load prerequisites ok"
