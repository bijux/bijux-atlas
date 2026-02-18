#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-obs-up"
PROFILE="${PROFILE:-${ATLAS_OBS_PROFILE:-kind}}"
if [ "$PROFILE" = "compose" ]; then
  PROFILE="local-compose"
fi
ATLAS_OBS_PROFILE="$PROFILE" exec ./ops/obs/scripts/install_pack.sh --profile "$PROFILE"
