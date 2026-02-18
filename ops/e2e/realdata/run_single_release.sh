#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
REAL_ROOT="${ATLAS_REALDATA_ROOT:-$ROOT/artifacts/real-datasets}"
export ATLAS_REALDATA_ROOT="$REAL_ROOT"

"$ROOT/scripts/fixtures/fetch-real-datasets.sh"
"$ROOT/ops/e2e/scripts/cleanup_store.sh"
"$ROOT/ops/datasets/scripts/publish_by_name.sh" real110
"$ROOT/ops/k8s/scripts/deploy_atlas.sh"
"$ROOT/ops/e2e/scripts/warmup.sh"
"$ROOT/ops/e2e/scripts/smoke_queries.sh"
"$ROOT/ops/e2e/realdata/verify_snapshots.sh"

echo "realdata single-release scenario passed"
