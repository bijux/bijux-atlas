#!/usr/bin/env bash
# owner: bijux-atlas-operations
# purpose: simulate dataset corruption and mark quarantine on detection.
# stability: public
# called-by: make ops-drill-corruption-dataset
set -euo pipefail
ROOT="$(CDPATH='' cd -- "$(dirname -- "$0")/../../.." && pwd)"
STORE_ROOT="${ATLAS_E2E_STORE_ROOT:-$ROOT/artifacts/e2e-store}"
QDIR="$ROOT/artifacts/e2e-datasets/quarantine"
mkdir -p "$QDIR"
target="$(find "$STORE_ROOT" -type f -name 'manifest.json' | head -n1 || true)"
[ -n "$target" ] || { echo "no dataset manifest found under $STORE_ROOT" >&2; exit 1; }
q="$QDIR/$(basename "$(dirname "$target")").bad"
if [ -f "$q" ]; then
  echo "quarantine marker present, skipping repeated retry: $q"
  exit 0
fi
printf '\n#corrupted\n' >> "$target"
if ! jq . "$target" >/dev/null 2>&1; then
  echo "$(date -u +%FT%TZ) $target" > "$q"
  echo "corruption detected and quarantined: $q"
  exit 0
fi
echo "corruption drill failed: target still valid json" >&2
exit 1
