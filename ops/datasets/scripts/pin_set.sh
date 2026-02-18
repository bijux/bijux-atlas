#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
OUT_DIR="$ROOT/artifacts/ops/cache"
mkdir -p "$OUT_DIR"

DATASETS="${DATASETS:-${ATLAS_PINNED_DATASETS:-}}"
if [ -z "$DATASETS" ]; then
  echo "usage: DATASETS=release/species/assembly[,..] make ops-cache-pin-set" >&2
  exit 2
fi

ENV_FILE="$OUT_DIR/pins.env"
printf 'ATLAS_PINNED_DATASETS=%q\n' "$DATASETS" > "$ENV_FILE"
echo "wrote $ENV_FILE"
echo "export ATLAS_PINNED_DATASETS='$DATASETS'"
