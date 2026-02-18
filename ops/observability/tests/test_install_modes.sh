#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/observability/tests/observability-test-lib.sh"
require_bin kubectl

# Minimal mode must always be accepted and never require CRDs.
ATLAS_OBS_MODE=minimal "$ROOT/ops/observability/scripts/install_obs_pack.sh"
"$ROOT/ops/observability/scripts/uninstall_obs_pack.sh"

# Full mode must fail fast when ServiceMonitor CRD is absent.
if ! kubectl api-resources | grep -q '^servicemonitors'; then
  if ATLAS_OBS_MODE=full "$ROOT/ops/observability/scripts/install_obs_pack.sh" >/dev/null 2>&1; then
    echo "full mode unexpectedly succeeded without ServiceMonitor CRD" >&2
    exit 1
  fi
else
  ATLAS_OBS_MODE=full "$ROOT/ops/observability/scripts/install_obs_pack.sh"
  "$ROOT/ops/observability/scripts/uninstall_obs_pack.sh"
fi

echo "observability install mode behavior passed"
