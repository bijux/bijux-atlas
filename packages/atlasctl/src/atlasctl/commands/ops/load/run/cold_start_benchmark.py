#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import time
import urllib.request
from pathlib import Path


def main() -> int:
    base_url = os.environ.get("BASE_URL", "http://127.0.0.1:8080")
    query = os.environ.get("QUERY", "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38")
    out_dir = Path(os.environ.get("OUT_DIR", "artifacts/benchmarks/cold-start"))
    out_dir.mkdir(parents=True, exist_ok=True)

    start = time.time_ns()
    with urllib.request.urlopen(f"{base_url}{query}", timeout=30) as resp:  # nosec B310
        code = getattr(resp, "status", 200)
        body = resp.read()
    end = time.time_ns()

    (out_dir / "response.json").write_bytes(body)
    payload = {"http_code": int(code), "cold_start_ms": int((end - start) / 1_000_000)}
    text = json.dumps(payload, separators=(",", ":")) + "\n"
    (out_dir / "result.json").write_text(text, encoding="utf-8")
    print(text, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
