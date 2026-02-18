#!/usr/bin/env python3
# Purpose: diff current metrics scrape against metrics contract and emit human report.
# Inputs: ops metrics contract + artifacts/ops/obs/metrics.prom
# Outputs: artifacts/ops/obs/metrics-drift.md and non-zero on drift violations.
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]
CONTRACT = ROOT / "ops/obs/contract/metrics-contract.json"
SCRAPE = ROOT / "artifacts/ops/obs/metrics.prom"
REPORT = ROOT / "artifacts/ops/obs/metrics-drift.md"


def parse_metric_names(text: str) -> set[str]:
    return set(re.findall(r"^((?:bijux|atlas)_[a-zA-Z0-9_]+)\{", text, flags=re.MULTILINE))


def main() -> int:
    ap = argparse.ArgumentParser()
    ap.add_argument("--enforce", action="store_true", help="fail if required metrics are missing")
    args = ap.parse_args()
    contract = json.loads(CONTRACT.read_text(encoding="utf-8"))
    required = set(contract.get("required_metrics", {}).keys())
    if not required:
        print("metrics contract missing required_metrics", file=sys.stderr)
        return 1
    if not SCRAPE.exists():
        print(f"missing metrics scrape: {SCRAPE}", file=sys.stderr)
        return 1

    observed = parse_metric_names(SCRAPE.read_text(encoding="utf-8", errors="replace"))
    missing = sorted(required - observed)
    extra = sorted(observed - required)

    REPORT.parent.mkdir(parents=True, exist_ok=True)
    lines = [
        "# Metrics Drift Report",
        "",
        f"- required metrics: `{len(required)}`",
        f"- observed metrics: `{len(observed)}`",
        f"- missing required metrics: `{len(missing)}`",
        f"- observed-but-not-required metrics: `{len(extra)}`",
        "",
        "## Missing Required Metrics",
    ]
    lines += [f"- `{m}`" for m in missing] if missing else ["- none"]
    lines += ["", "## Observed But Not Required"]
    lines += [f"- `{m}`" for m in extra] if extra else ["- none"]
    REPORT.write_text("\n".join(lines) + "\n", encoding="utf-8")

    if missing and args.enforce:
        print(f"metrics drift failed: missing {len(missing)} required metrics", file=sys.stderr)
        print(f"report: {REPORT}", file=sys.stderr)
        return 1
    if missing:
        print(f"metrics drift warning: missing {len(missing)} required metrics; report: {REPORT}")
    else:
        print(f"metrics drift check passed; report: {REPORT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
