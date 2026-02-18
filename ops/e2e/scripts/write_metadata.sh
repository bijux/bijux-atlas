#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
# shellcheck source=ops/_lib/common.sh
source "$ROOT/ops/_lib/common.sh"
out="${1:-$OPS_RUN_DIR}"
ops_write_metadata "$out"
echo "wrote $out/metadata.json"
