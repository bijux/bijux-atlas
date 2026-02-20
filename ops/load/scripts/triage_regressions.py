#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def _measured(results: Path) -> dict[str, float]:
    out: dict[str, float] = {}
    for f in sorted(results.glob("*.summary.json")):
        data = json.loads(f.read_text(encoding="utf-8"))
        out[f.stem.replace(".summary", "")] = float(
            data.get("metrics", {}).get("http_req_duration", {}).get("values", {}).get("p(95)", 0.0)
        )
    return out


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--profile", default="local")
    p.add_argument("--results", default="artifacts/perf/results")
    args = p.parse_args()

    baseline = json.loads((ROOT / "configs/ops/perf/baselines" / f"{args.profile}.json").read_text(encoding="utf-8"))
    measured = _measured((ROOT / args.results).resolve())
    rows = []
    for r in baseline.get("rows", []):
        suite = str(r.get("suite", ""))
        if suite not in measured:
            continue
        delta = measured[suite] - float(r.get("p95_ms", 0))
        rows.append((delta, suite, float(r.get("p95_ms", 0)), measured[suite]))
    rows.sort(reverse=True)
    print("top regressions by endpoint/suite (p95 ms):")
    for delta, suite, base, now in rows[:10]:
        print(f"- {suite}: baseline={base:.2f} now={now:.2f} delta={delta:+.2f}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
