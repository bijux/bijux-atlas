#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"
require_bin kubectl

# local-compose profile should be accepted when compose is installed.
if (command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1) || command -v docker-compose >/dev/null 2>&1; then
  ATLAS_OBS_PROFILE=local-compose "$ROOT/ops/obs/scripts/install_pack.sh"
  ATLAS_OBS_PROFILE=local-compose "$ROOT/ops/obs/scripts/install_pack.sh"
  ATLAS_OBS_PROFILE=local-compose "$ROOT/ops/obs/scripts/verify_pack.sh"
  ATLAS_OBS_PROFILE=local-compose "$ROOT/ops/obs/scripts/uninstall_pack.sh"
  ATLAS_OBS_PROFILE=local-compose "$ROOT/ops/obs/scripts/install_pack.sh"
  ATLAS_OBS_PROFILE=local-compose "$ROOT/ops/obs/scripts/uninstall_pack.sh"
else
  echo "local-compose profile skipped: docker compose unavailable"
fi

# Kind profile must always be accepted and never require ServiceMonitor CRD.
ATLAS_OBS_PROFILE=kind "$ROOT/ops/obs/scripts/install_pack.sh"
"$ROOT/ops/obs/scripts/uninstall_pack.sh"

# Cluster profile must fail fast when ServiceMonitor CRD is absent.
if ! kubectl api-resources | grep -q '^servicemonitors'; then
  if ATLAS_OBS_PROFILE=cluster "$ROOT/ops/obs/scripts/install_pack.sh" >/dev/null 2>&1; then
    echo "cluster profile unexpectedly succeeded without ServiceMonitor CRD" >&2
    exit 1
  fi
else
  ATLAS_OBS_PROFILE=cluster "$ROOT/ops/obs/scripts/install_pack.sh"
  "$ROOT/ops/obs/scripts/uninstall_pack.sh"
fi

echo "observability profile behavior passed"
