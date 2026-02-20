#!/usr/bin/env python3
# owner: platform
# purpose: enforce p95 regression threshold against versioned baseline.
# stability: public
# called-by: make stack-full
# Purpose: compare measured k6 p95 metrics to versioned baselines with bounded regression budget.
# Inputs: --baseline-profile, --max-p95-regression, and k6 summary JSON files under artifacts/perf/results.
# Outputs: exit 0 on pass; exit 1 and write artifacts/stack-report/regression.txt on violations.

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def load_measured(results_dir: Path) -> dict[str, float]:
    out: dict[str, float] = {}
    for summary in sorted(results_dir.glob("*.summary.json")):
        data = json.loads(summary.read_text())
        p95 = float(data.get("metrics", {}).get("http_req_duration", {}).get("values", {}).get("p(95)", 0.0))
        out[summary.name.replace(".summary.json", "")] = p95
    return out


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--baseline-profile", default="local")
    parser.add_argument("--max-p95-regression", type=float, default=0.15)
    parser.add_argument("--results", default="artifacts/perf/results")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[3]
    baseline_file = root / "configs/ops/perf/baselines" / f"{args.baseline_profile}.json"
    if not baseline_file.exists():
        print(f"baseline file missing: {baseline_file}", file=sys.stderr)
        return 1

    baseline = json.loads(baseline_file.read_text())
    rows = baseline.get("rows", [])
    if not rows:
        print(f"baseline rows missing in {baseline_file}", file=sys.stderr)
        return 1

    measured = load_measured(root / args.results)
    if not measured:
        print(f"measured summaries missing in {args.results}", file=sys.stderr)
        return 1

    violations: list[str] = []
    for row in rows:
        suite = row.get("suite")
        if suite not in measured:
            continue
        baseline_p95 = float(row.get("p95_ms", 0.0))
        allowed = baseline_p95 * (1.0 + args.max_p95_regression)
        if measured[suite] > allowed:
            violations.append(
                f"{suite}: measured p95={measured[suite]:.2f}ms exceeds +{args.max_p95_regression*100:.0f}% baseline ({allowed:.2f}ms)"
            )

    if violations:
        out = root / "artifacts/stack-report/regression.txt"
        out.parent.mkdir(parents=True, exist_ok=True)
        out.write_text("\n".join(violations) + "\n")
        print("p95 regression check failed", file=sys.stderr)
        for line in violations:
            print(f"- {line}", file=sys.stderr)
        return 1

    print("p95 regression check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
