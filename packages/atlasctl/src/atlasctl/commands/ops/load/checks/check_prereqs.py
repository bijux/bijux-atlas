#!/usr/bin/env python3
from __future__ import annotations

import shutil
import sys
import urllib.request
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def main() -> int:
    del _repo_root  # interface parity; root not otherwise needed here
    base_url = (
        __import__("os").environ.get("ATLAS_BASE_URL", "http://127.0.0.1:18080").rstrip("/")
    )
    for tool in ("k6", "curl"):
        if shutil.which(tool) is None:
            print(f"{tool} required", file=sys.stderr)
            return 1
    try:
        with urllib.request.urlopen(f"{base_url}/healthz", timeout=5) as resp:  # nosec B310
            if getattr(resp, "status", 200) >= 400:
                raise RuntimeError(f"http {resp.status}")
    except Exception:
        print(f"atlas endpoint not reachable at {base_url}", file=sys.stderr)
        return 1
    print("load prerequisites ok")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
