#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../.." && pwd)"
cd "$ROOT"
. "$ROOT/ops/_lib/common.sh"
ops_env_load
ops_entrypoint_start "ops-clean"
ops_version_guard python3
days="$(python3 - <<'PY'
import json
from pathlib import Path
p = Path("configs/ops/artifact-retention.json")
if p.exists():
    print(int(json.loads(p.read_text(encoding="utf-8")).get("ops_retention_days", 7)))
else:
    print(7)
PY
)"
days="${OPS_RETENTION_DAYS:-$days}"
evidence_days="$(python3 - <<'PY'
import json
from pathlib import Path
p = Path("configs/ops/artifact-retention.json")
if p.exists():
    print(int(json.loads(p.read_text(encoding="utf-8")).get("evidence_retention_days", 14)))
else:
    print(14)
PY
)"
if [ "${OPS_EVIDENCE_RETENTION_DAYS+x}" = "x" ]; then
  evidence_days="${OPS_EVIDENCE_RETENTION_DAYS}"
fi
find artifacts/ops -mindepth 1 -maxdepth 1 -type d -mtime +"$days" -exec rm -rf {} + 2>/dev/null || true
find ops/_evidence/make -mindepth 1 -maxdepth 1 -type d -name "atlas-ops-*" -mtime +"$evidence_days" -exec rm -rf {} + 2>/dev/null || true
rm -rf artifacts/perf/results artifacts/e2e-datasets artifacts/e2e-store
