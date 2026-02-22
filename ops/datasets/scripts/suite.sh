#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"

SUITE="${1:-verify}"
case "$SUITE" in
  verify)
    python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/datasets/fetch_and_verify.py"
    ;;
  qc)
    python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/datasets/dataset_qc.py"
    ;;
  promotion)
    python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/datasets/promotion_sim.py"
    ;;
  corruption)
    python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/datasets/corruption_drill.py"
    ;;
  *)
    echo "unknown dataset suite: $SUITE (expected: verify|qc|promotion|corruption)" >&2
    exit 2
    ;;
esac
