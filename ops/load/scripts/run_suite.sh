#!/usr/bin/env sh
# Purpose: script interface entrypoint.
# Inputs: command-line args and repository files/env as documented by caller.
# Outputs: exit status and deterministic stdout/stderr or generated artifacts.
set -eu

ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
INPUT="${1:?suite or scenario required, e.g. mixed-80-20.js or mixed.json}"
OUT_DIR="${2:-$ROOT/artifacts/perf/results}"
BASE_URL="${ATLAS_BASE_URL:-${BASE_URL:-http://127.0.0.1:18080}}"
API_KEY="${ATLAS_API_KEY:-}"
DATASET_HASH="${ATLAS_DATASET_HASH:-unknown}"
DATASET_RELEASE="${ATLAS_DATASET_RELEASE:-unknown}"
IMAGE_DIGEST="${ATLAS_IMAGE_DIGEST:-unknown}"
GIT_SHA="${GITHUB_SHA:-$(git -C "$ROOT" rev-parse --short=12 HEAD 2>/dev/null || echo unknown)}"
POLICY_HASH="${ATLAS_POLICY_HASH:-$(shasum -a 256 "$ROOT/configs/policy/policy.json" 2>/dev/null | awk '{print $1}' || echo unknown)}"

mkdir -p "$OUT_DIR"
if printf '%s' "$INPUT" | grep -q '\.json$'; then
  SCENARIO_PATH="$INPUT"
  case "$SCENARIO_PATH" in
    /*) ;;
    *) SCENARIO_PATH="$ROOT/ops/load/scenarios/$SCENARIO_PATH" ;;
  esac
  SUITE="$(python3 - <<PY
import json
from pathlib import Path
p=Path("$SCENARIO_PATH")
d=json.loads(p.read_text())
print(d.get("suite",""))
PY
)"
  [ -n "$SUITE" ] || { echo "scenario has no suite: $SCENARIO_PATH" >&2; exit 2; }
  NAME="$(basename "$SCENARIO_PATH" .json)"
else
  SUITE="$INPUT"
  NAME="${SUITE%.js}"
fi

SUMMARY_JSON="$OUT_DIR/${NAME}.summary.json"

if command -v k6 >/dev/null 2>&1; then
  BASE_URL="$BASE_URL" ATLAS_API_KEY="$API_KEY" k6 run --summary-export "$SUMMARY_JSON" "$ROOT/ops/load/k6/suites/$SUITE"
else
  docker run --rm --network host \
    -e BASE_URL="$BASE_URL" \
    -e ATLAS_API_KEY="$API_KEY" \
    -v "$ROOT:/work" -w /work \
    grafana/k6:0.49.0 run --summary-export "$SUMMARY_JSON" "ops/load/k6/suites/$SUITE"
fi

cat > "${OUT_DIR}/${NAME}.meta.json" <<JSON
{"suite":"$INPUT","resolved_suite":"$SUITE","git_sha":"$GIT_SHA","image_digest":"$IMAGE_DIGEST","dataset_hash":"$DATASET_HASH","dataset_release":"$DATASET_RELEASE","policy_hash":"$POLICY_HASH","base_url":"$BASE_URL"}
JSON

echo "suite complete: $INPUT ($SUITE) -> $SUMMARY_JSON"
