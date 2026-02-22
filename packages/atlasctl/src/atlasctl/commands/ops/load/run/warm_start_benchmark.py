#!/usr/bin/env python3
from __future__ import annotations

import json
import os
import time
import urllib.request
from pathlib import Path


def _fetch(url: str) -> tuple[int, bytes]:
    with urllib.request.urlopen(url, timeout=30) as resp:  # nosec B310
        return int(getattr(resp, "status", 200)), resp.read()


def main() -> int:
    base_url = os.environ.get("BASE_URL", "http://127.0.0.1:8080")
    query = os.environ.get("QUERY", "/v1/genes/count?release=110&species=homo_sapiens&assembly=GRCh38")
    out_dir = Path(os.environ.get("OUT_DIR", "artifacts/benchmarks/warm-start"))
    warm_requests = int(os.environ.get("WARM_REQUESTS", "5"))
    out_dir.mkdir(parents=True, exist_ok=True)
    url = f"{base_url}{query}"

    for _ in range(max(0, warm_requests)):
        _fetch(url)

    start = time.time_ns()
    code, body = _fetch(url)
    end = time.time_ns()
    (out_dir / "response.json").write_bytes(body)
    payload = {
        "http_code": int(code),
        "warm_start_ms": int((end - start) / 1_000_000),
        "warm_requests": warm_requests,
    }
    text = json.dumps(payload, separators=(",", ":")) + "\n"
    (out_dir / "result.json").write_text(text, encoding="utf-8")
    print(text, end="")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
