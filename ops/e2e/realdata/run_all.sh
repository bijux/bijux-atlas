#!/usr/bin/env sh
set -eu

ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
DATASET_SOURCE="${REALDATA_SOURCE:-ops/datasets/real-datasets.json}"
[ -f "$ROOT/$DATASET_SOURCE" ] || { echo "missing declared dataset source: $DATASET_SOURCE" >&2; exit 2; }

"$ROOT/ops/e2e/realdata/run_single_release.sh"
"$ROOT/ops/e2e/realdata/run_two_release_diff.sh"
"$ROOT/ops/e2e/realdata/schema_evolution.sh"
"$ROOT/ops/e2e/realdata/upgrade_drill.sh"
"$ROOT/ops/e2e/realdata/rollback_drill.sh"

echo "realdata suite passed"
