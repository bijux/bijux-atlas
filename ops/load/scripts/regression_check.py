#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
BUDGETS = ROOT / "configs/ops/budgets.json"


def load_measured(results_dir: Path) -> dict[str, float]:
    out: dict[str, float] = {}
    for summary in sorted(results_dir.glob("*.summary.json")):
        data = json.loads(summary.read_text(encoding="utf-8"))
        p95 = float(data.get("metrics", {}).get("http_req_duration", {}).get("values", {}).get("p(95)", 0.0))
        out[summary.stem.replace(".summary", "")] = p95
    return out


def main() -> int:
    p = argparse.ArgumentParser()
    p.add_argument("--profile", default="local")
    p.add_argument("--results", default="artifacts/perf/results")
    args = p.parse_args()

    budgets = json.loads(BUDGETS.read_text(encoding="utf-8"))
    max_ratio = float(budgets.get("k6_latency", {}).get("max_p95_regression_ratio", 0.15))
    baseline_file = ROOT / "configs/ops/perf/baselines" / f"{args.profile}.json"
    if not baseline_file.exists():
        print(f"baseline file missing: {baseline_file}", file=sys.stderr)
        return 1

    baseline = json.loads(baseline_file.read_text(encoding="utf-8"))
    measured = load_measured((ROOT / args.results).resolve())
    if not measured:
        print(f"measured summaries missing in {args.results}", file=sys.stderr)
        return 1

    violations: list[str] = []
    for row in baseline.get("rows", []):
        suite = str(row.get("suite", ""))
        if suite not in measured:
            continue
        base = float(row.get("p95_ms", 0.0))
        allowed = base * (1.0 + max_ratio)
        if measured[suite] > allowed:
            violations.append(
                f"{suite}: measured p95={measured[suite]:.2f}ms exceeds +{max_ratio*100:.0f}% baseline ({allowed:.2f}ms)"
            )

    run_id = (ROOT / "ops/_evidence/latest-run-id.txt").read_text(encoding="utf-8").strip() if (ROOT / "ops/_evidence/latest-run-id.txt").exists() else "manual"
    out = ROOT / "ops/_evidence/perf" / run_id / "regression-check.txt"
    out.parent.mkdir(parents=True, exist_ok=True)
    if violations:
        out.write_text("\n".join(violations) + "\n", encoding="utf-8")
        print("perf regression check failed", file=sys.stderr)
        for v in violations:
            print(f"- {v}", file=sys.stderr)
        return 1

    out.write_text("pass\n", encoding="utf-8")
    print("perf regression check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
