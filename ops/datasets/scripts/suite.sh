#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"

SUITE="${1:-verify}"
case "$SUITE" in
  verify)
    "$ROOT/ops/datasets/scripts/sh/fetch_and_verify.sh"
    ;;
  qc)
    "$ROOT/ops/datasets/scripts/sh/dataset_qc.sh"
    ;;
  promotion)
    python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/datasets/promotion_sim.py"
    ;;
  corruption)
    "$ROOT/ops/datasets/scripts/sh/corruption_drill.sh"
    ;;
  *)
    echo "unknown dataset suite: $SUITE (expected: verify|qc|promotion|corruption)" >&2
    exit 2
    ;;
esac
