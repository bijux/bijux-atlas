#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]


def main() -> int:
    script = r"""
set -euo pipefail
ROOT="$(pwd)"
. "$ROOT/ops/obs/tests/observability-test-lib.sh"
require_bin kubectl

if [ "${OBS_SKIP_LOCAL_COMPOSE:-0}" = "1" ]; then
  echo "local-compose profile skipped: OBS_SKIP_LOCAL_COMPOSE=1"
elif (command -v docker >/dev/null 2>&1 && docker compose version >/dev/null 2>&1) || command -v docker-compose >/dev/null 2>&1; then
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

ATLAS_OBS_PROFILE=kind python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/install_pack.py"
python3 "$ROOT/packages/atlasctl/src/atlasctl/commands/ops/observability/uninstall_pack.py"

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
"""
    return subprocess.run(["bash", "-lc", script], cwd=ROOT).returncode


if __name__ == "__main__":
    raise SystemExit(main())
