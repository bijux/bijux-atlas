#!/usr/bin/env python3
from __future__ import annotations

import subprocess
from pathlib import Path


def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


def _fetch_metrics(base_url: str) -> str:
    try:
        proc = subprocess.run(["curl", "-fsS", f"{base_url}/metrics"], text=True, capture_output=True, check=False)
        return proc.stdout if proc.returncode == 0 else ""
    except Exception:
        return ""


def _metric_last_value(metrics: str, prefix: str) -> int:
    last = None
    for line in metrics.splitlines():
        if not line.startswith(prefix):
            continue
        try:
            last = int(float(line.split()[-1]))
        except Exception:
            continue
    return int(last or 0)


def main() -> int:
    root = _repo_root()
    base_url = __import__('os').environ.get("ATLAS_BASE_URL", "http://127.0.0.1:18080")
    metrics = _fetch_metrics(base_url)
    hits = _metric_last_value(metrics, "bijux_dataset_hits")
    misses = _metric_last_value(metrics, "bijux_dataset_misses")
    try:
        proc = subprocess.run(["du", "-sk", str(root / "artifacts/e2e-store")], text=True, capture_output=True, check=False)
        usage = int(proc.stdout.split()[0]) * 1024 if proc.returncode == 0 and proc.stdout.strip() else 0
    except Exception:
        usage = 0
    total = hits + misses
    ratio = (hits / total) if total > 0 else 0.0
    print(f"cache_hits={hits}")
    print(f"cache_misses={misses}")
    print(f"cache_hit_ratio={ratio:.6f}")
    print(f"cache_disk_bytes={usage}")
    out_dir = root / "artifacts/ops"
    out_dir.mkdir(parents=True, exist_ok=True)
    (out_dir / "cache-status.json").write_text(
        f'{{"cache_hits":{hits},"cache_misses":{misses},"cache_hit_ratio":{ratio:.6f},"cache_disk_bytes":{usage}}}\n',
        encoding="utf-8",
    )
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
