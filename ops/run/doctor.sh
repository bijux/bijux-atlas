#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
./ops/run/prereqs.sh
python3 ./scripts/layout/check_tool_versions.py || true
make -s ops-env-print || true
