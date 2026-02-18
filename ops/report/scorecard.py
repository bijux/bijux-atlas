#!/usr/bin/env python3
# Purpose: compute operational readiness scorecard from ops run artifacts.
# Inputs: --run-dir and --out
# Outputs: markdown scorecard and non-zero exit if required checks are missing.
from __future__ import annotations

import argparse
from pathlib import Path


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-dir", required=True)
    parser.add_argument("--out", required=True)
    args = parser.parse_args()

    run_dir = Path(args.run_dir)
    checks = {
        "smoke_report": (run_dir / "smoke" / "report.md").exists(),
        "k8s_results": (run_dir / "k8s" / "test-results.json").exists(),
        "perf_results": (run_dir / "perf" / "results").exists() or (run_dir / "perf").exists(),
        "observability_metrics": (run_dir / "observability" / "metrics.prom").exists(),
        "ops_report": (run_dir / "report.json").exists(),
    }
    passed = sum(1 for ok in checks.values() if ok)
    total = len(checks)
    score = int((passed / total) * 100)

    lines = [
        "# Operational Readiness Scorecard",
        "",
        f"- Run dir: `{run_dir}`",
        f"- Score: `{score}`",
        "",
        "## Checks",
        "",
    ]
    for name, ok in checks.items():
        lines.append(f"- {name}: `{'PASS' if ok else 'FAIL'}`")

    out = Path(args.out)
    out.parent.mkdir(parents=True, exist_ok=True)
    out.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(out)
    return 0 if all(checks.values()) else 1


if __name__ == "__main__":
    raise SystemExit(main())
