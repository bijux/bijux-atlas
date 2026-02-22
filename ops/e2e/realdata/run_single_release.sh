#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
REAL_ROOT="${ATLAS_REALDATA_ROOT:-$ROOT/artifacts/real-datasets}"
export ATLAS_REALDATA_ROOT="$REAL_ROOT"

python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/datasets/fixtures/fetch_real_datasets.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/cleanup_store.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/datasets/publish_by_name.py" real110
(
  cd "$ROOT"
  ./bin/atlasctl ops deploy --report text apply
)
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/warmup.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/e2e/runtime/smoke_queries.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/e2e/realdata/verify_snapshots.py"

echo "realdata single-release scenario passed"
