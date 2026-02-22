#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
REAL_ROOT="${ATLAS_REALDATA_ROOT:-$ROOT/artifacts/real-datasets}"
export ATLAS_REALDATA_ROOT="$REAL_ROOT"

"$ROOT/ops/datasets/scripts/fixtures/fetch-real-datasets.sh"
"$ROOT/ops/e2e/runner/cleanup_store.sh"
"$ROOT/ops/datasets/scripts/publish_by_name.sh" real110
(
  cd "$ROOT"
  ./bin/atlasctl ops deploy --report text apply
)
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warmup.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/smoke_queries.py"
"$ROOT/ops/e2e/realdata/verify_snapshots.sh"

echo "realdata single-release scenario passed"
