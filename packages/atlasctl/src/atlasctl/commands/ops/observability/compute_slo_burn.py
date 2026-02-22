#!/usr/bin/env python3
# Purpose: compute SLO burn from k6 score artifacts and metrics snapshot.
from __future__ import annotations

import argparse
import json
import re
from pathlib import Path

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in cur.parents:
        if all((parent / marker).exists() for marker in ("makefiles", "packages", "configs", "ops")):
            return parent
    raise RuntimeError("unable to resolve repo root")


ROOT = _repo_root()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--k6", default=str(ROOT / "artifacts/ops/e2e/k6/score.md"))
    parser.add_argument("--metrics", default=str(ROOT / "artifacts/ops/obs/metrics.prom"))
    parser.add_argument("--out", default=str(ROOT / "artifacts/ops/obs/slo-burn.json"))
    args = parser.parse_args()

    k6 = Path(args.k6)
    metrics = Path(args.metrics)
    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)

    violations = 1 if k6.exists() and "Violations" in k6.read_text(encoding="utf-8") else 0

    store_open = 0
    metrics_text = metrics.read_text(encoding="utf-8") if metrics.exists() else ""
    if "bijux_store_breaker_open" in metrics_text:
        store_open = 1

    total = 0.0
    errors_5xx = 0.0
    for line in metrics_text.splitlines():
        if not line.startswith("bijux_http_requests_total{"):
            continue
        try:
            val = float(line.rsplit(" ", 1)[-1])
        except Exception:
            continue
        total += val
        m = re.search(r'status="([0-9]{3})"', line)
        if m and m.group(1).startswith("5"):
            errors_5xx += val

    error_rate = (errors_5xx / total) if total > 0 else 0.0
    burn_rate = (error_rate / 0.005) if total > 0 else 0.0
    burn_exceeded = bool(violations or burn_rate > 1.0)
    payload = {
        "schema_version": 1,
        "k6_violations": violations,
        "store_circuit_open_seen": store_open,
        "http_total_requests": total,
        "http_5xx_requests": errors_5xx,
        "error_rate": error_rate,
        "burn_rate": burn_rate,
        "burn_exceeded": burn_exceeded,
    }
    out.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")
    print(out)
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
