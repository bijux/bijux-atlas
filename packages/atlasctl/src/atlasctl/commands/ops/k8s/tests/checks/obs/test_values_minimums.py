#!/usr/bin/env python3
from __future__ import annotations

import re
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    root = _repo_root()
    text = (root / "ops/k8s/charts/bijux-atlas/values.yaml").read_text(encoding="utf-8")
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
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
