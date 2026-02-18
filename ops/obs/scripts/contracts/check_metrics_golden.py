#!/usr/bin/env python3
# Purpose: compare current metrics scrape against golden snapshot with tolerance.
# Inputs: artifacts/ops/obs/metrics.prom and ops/obs/contract/metrics.golden.prom
# Outputs: non-zero when golden-drift exceeds tolerated bounds.
from __future__ import annotations

import math
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CURRENT = ROOT / "artifacts/ops/obs/metrics.prom"
GOLDEN = ROOT / "ops/obs/contract/metrics.golden.prom"

# Relative tolerance by metric name.
REL_TOL = {
    "bijux_http_request_latency_p95_seconds": 0.50,
    "bijux_http_request_size_p95_bytes": 0.50,
    "bijux_http_response_size_p95_bytes": 0.50,
    "bijux_sqlite_query_latency_p95_seconds": 0.50,
    "bijux_store_fetch_latency_p95_seconds": 0.60,
    "bijux_store_open_p95_seconds": 0.60,
    "bijux_store_download_p95_seconds": 0.60,
}


def parse_metrics(path: Path) -> dict[tuple[str, str], float]:
    text = path.read_text(encoding="utf-8", errors="replace")
    out: dict[tuple[str, str], float] = {}
    for line in text.splitlines():
        if not line or line.startswith("#"):
            continue
        m = re.match(r"^((?:bijux|atlas)_[a-zA-Z0-9_]+)(\{[^}]*\})\s+([-+eE0-9.]+)$", line)
        if not m:
            continue
        name, labels, raw = m.groups()
        try:
            out[(name, labels)] = float(raw)
        except ValueError:
            continue
    return out


def rel_diff(a: float, b: float) -> float:
    denom = max(abs(a), abs(b), 1e-9)
    return abs(a - b) / denom


def main() -> int:
    if not CURRENT.exists() or not GOLDEN.exists():
        print("missing current or golden metrics snapshot", file=sys.stderr)
        return 1

    cur = parse_metrics(CURRENT)
    old = parse_metrics(GOLDEN)

    # Compare common series keys and report additive/missing drift.
    missing_series = sorted(set(old) - set(cur))
    extra_series = sorted(set(cur) - set(old))

    violations: list[str] = []
    for key in sorted(set(cur) & set(old)):
        name, labels = key
        tol = REL_TOL.get(name)
        if tol is None:
            continue
        d = rel_diff(cur[key], old[key])
        if d > tol and not (math.isnan(cur[key]) or math.isnan(old[key])):
            violations.append(
                f"{name}{labels} relative-diff={d:.3f} exceeds tolerance={tol:.3f} (current={cur[key]:.6g} golden={old[key]:.6g})"
            )

    if missing_series:
        print("metrics golden warning: missing expected series", file=sys.stderr)
        for name, labels in missing_series[:50]:
            print(f"- {name}{labels}", file=sys.stderr)

    # Extra series are reported but non-fatal (additive evolution allowed).
    if extra_series:
        print(f"metrics golden warning: {len(extra_series)} additive series observed", file=sys.stderr)

    if violations:
        print("metrics golden check failed: tolerance violations", file=sys.stderr)
        for v in violations[:50]:
            print(f"- {v}", file=sys.stderr)
        return 1

    print("metrics golden check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
