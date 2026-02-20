#!/usr/bin/env bash
set -euo pipefail
ROOT="$(CDPATH= cd -- "$(dirname -- "$0")/../../../.." && pwd)"
cd "$ROOT"

python3 - <<'PY'
import re
from pathlib import Path

text = Path("ops/k8s/charts/bijux-atlas/values.yaml").read_text(encoding="utf-8")
def get(pattern: str, default: str = "") -> str:
    m = re.search(pattern, text, flags=re.MULTILINE)
    return m.group(1) if m else default

replicas = int(get(r"^replicaCount:\s*([0-9]+)", "0"))
request_timeout = int(get(r"^\s*requestTimeoutMs:\s*([0-9]+)", "0"))
sql_timeout = int(get(r"^\s*sqlTimeoutMs:\s*([0-9]+)", "0"))
cpu_req = get(r"^\s*cpu:\s*\"([0-9]+m)\"", "")
mem_req = get(r"^\s*memory:\s*\"([0-9]+Mi)\"", "")
if replicas < 1:
    raise SystemExit("values minimums failed: replicaCount must be >= 1")
if request_timeout < 1000:
    raise SystemExit("values minimums failed: requestTimeoutMs must be >= 1000")
if sql_timeout < 200:
    raise SystemExit("values minimums failed: sqlTimeoutMs must be >= 200")
if not cpu_req or not mem_req:
    raise SystemExit("values minimums failed: resources requests cpu/memory must be set")
print("values minimums contract passed")
PY
