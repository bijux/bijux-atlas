#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
source "$ROOT/_lib/common.sh"
ops_init_run_id

BASE="${ATLAS_BASE_URL:-http://127.0.0.1:8080}"
DATASETS="${DATASETS:-${ATLAS_STARTUP_WARMUP:-}}"
if [ -z "$DATASETS" ]; then
  echo "usage: DATASETS=release/species/assembly[,..] make ops-warm-datasets" >&2
  exit 2
fi

OUT="$(ops_artifact_dir warm-datasets)"
IFS=',' read -r -a items <<<"$DATASETS"
for ds in "${items[@]}"; do
  ds="$(echo "$ds" | xargs)"
  [ -z "$ds" ] && continue
  IFS='/' read -r rel species assembly <<<"$ds"
  test -n "$rel" && test -n "$species" && test -n "$assembly"
  url="$BASE/v1/genes?release=$rel&species=$species&assembly=$assembly&gene_id=GENE1"
  ops_retry 3 1 curl -fsS "$url" >"$OUT/${rel}_${species}_${assembly}.json" || true
  echo "warmed $ds"
done
