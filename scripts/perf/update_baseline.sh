#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
PROFILE="${1:-${ATLAS_PERF_BASELINE_PROFILE:-local}}"
SRC="$ROOT/artifacts/perf/baseline.json"
DST="$ROOT/ops/load/baselines/${PROFILE}.json"

[ -f "$SRC" ] || { echo "missing baseline source: $SRC" >&2; exit 1; }
cp "$SRC" "$DST"

echo "baseline updated: $DST"
git -C "$ROOT" diff -- "$DST" || true
echo "baseline update policy: set ATLAS_BASELINE_APPROVED=1 for commit gate"
