#!/usr/bin/env bash
set -euo pipefail

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
source "$ROOT/ops/_lib/common.sh"

release="${ATLAS_DATASET_RELEASE:-110}"
species="${ATLAS_DATASET_SPECIES:-homo_sapiens}"
assembly="${ATLAS_DATASET_ASSEMBLY:-GRCh38}"
base="${ATLAS_E2E_BASE_URL:-http://127.0.0.1:${ATLAS_E2E_LOCAL_PORT:-18080}}"
out="$(ops_artifact_dir warm-shards)"

curl -fsS "$base/v1/genes?release=$release&species=$species&assembly=$assembly&region=chr1:1-100000&limit=10" >"$out/chr1.json"
curl -fsS "$base/v1/genes?release=$release&species=$species&assembly=$assembly&region=chr2:1-100000&limit=10" >"$out/chr2.json"

echo "shard warmup completed"
