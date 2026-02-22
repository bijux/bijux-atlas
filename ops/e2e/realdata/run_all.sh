#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
DATASET_SOURCE="${REALDATA_SOURCE:-ops/datasets/real-datasets.json}"
[ -f "$ROOT/$DATASET_SOURCE" ] || { echo "missing declared dataset source: $DATASET_SOURCE" >&2; exit 2; }

exec "$ROOT/atlasctl ops e2e run" --suite realdata "$@"
