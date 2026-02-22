#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../.." && pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"
require_bin kubectl

# local-compose profile should be accepted when compose is installed.
if [ "${OBS_SKIP_LOCAL_COMPOSE:-0}" = "1" ]; then
  echo "local-compose profile skipped: OBS_SKIP_LOCAL_COMPOSE=1"
elif (command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1) || command -v docker-compose >/dev/null 2>&1; then
  # Clean up stale local-compose services from interrupted runs before asserting idempotency.
  ATLAS_OBS_PROFILE=local-compose python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/uninstall_pack.py" >/dev/null 2>&1 || true
  ATLAS_OBS_PROFILE=local-compose python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/install_pack.py"
  ATLAS_OBS_PROFILE=local-compose python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/install_pack.py"
  ATLAS_OBS_PROFILE=local-compose python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/verify_pack.py"
  ATLAS_OBS_PROFILE=local-compose python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/uninstall_pack.py"
  ATLAS_OBS_PROFILE=local-compose python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/install_pack.py"
  ATLAS_OBS_PROFILE=local-compose python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/uninstall_pack.py"
else
  echo "local-compose profile skipped: docker compose unavailable"
fi

# Kind profile must always be accepted and never require ServiceMonitor CRD.
ATLAS_OBS_PROFILE=kind python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/install_pack.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/uninstall_pack.py"

# Cluster profile must fail fast when ServiceMonitor CRD is absent.
if ! kubectl api-resources | grep -q '^servicemonitors'; then
  if ATLAS_OBS_PROFILE=cluster python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/install_pack.py" >/dev/null 2>&1; then
    echo "cluster profile unexpectedly succeeded without ServiceMonitor CRD" >&2
    exit 1
  fi
else
  ATLAS_OBS_PROFILE=cluster python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/install_pack.py"
  python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/uninstall_pack.py"
fi

echo "observability profile behavior passed"
