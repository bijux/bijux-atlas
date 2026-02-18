#!/usr/bin/env python3
# owner: platform
# purpose: validate stack-full report bundle contract and required non-empty artifacts.
# stability: public
# called-by: make stack-full
# Purpose: enforce the stack-full report contract and required non-empty evidence artifacts.
# Inputs: --report-dir and --schema paths.
# Outputs: exit 0 on valid bundle; exit 1 with a deterministic failure reason.

from __future__ import annotations

import argparse
import json
import sys
from pathlib import Path


def fail(msg: str) -> int:
    print(msg, file=sys.stderr)
    return 1


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--report-dir", default="artifacts/stack-report")
    parser.add_argument("--schema", default="ops/_schemas/report/stack-contract.schema.json")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[3]
    report_dir = root / args.report_dir
    summary_path = report_dir / "pass-fail-summary.json"

    if not Path(root / args.schema).exists():
        return fail(f"missing schema: {args.schema}")
    if not summary_path.exists():
        return fail(f"missing summary: {summary_path}")

    summary = json.loads(summary_path.read_text())
    required_top = ["schema_version", "stack_version_hash", "status", "run_id", "generated_at_utc", "artifacts"]
    for key in required_top:
        if key not in summary:
            return fail(f"summary missing key: {key}")

    required_artifacts = [
        "helm_values_used",
        "k6_summary",
        "metrics_snapshot",
        "trace_snapshot",
        "dashboard_screenshot",
        "logs_excerpt",
        "rendered_manifests",
        "pass_fail_summary",
    ]
    artifacts = summary.get("artifacts", {})
    for key in required_artifacts:
        if key not in artifacts:
            return fail(f"summary.artifacts missing key: {key}")
        p = root / artifacts[key]
        if not p.exists():
            return fail(f"missing artifact: {artifacts[key]}")

    # Hard fail requirements from stack-full contract.
    if (root / artifacts["metrics_snapshot"]).stat().st_size == 0:
        return fail("metrics snapshot is empty")
    if (root / artifacts["trace_snapshot"]).stat().st_size == 0:
        return fail("trace snapshot is empty")
    if (root / artifacts["rendered_manifests"]).stat().st_size == 0:
        return fail("rendered manifests snapshot is empty")
    if not json.loads((root / artifacts["k6_summary"]).read_text()).get("summaries"):
        return fail("k6 summary missing entries")

    print("stack report contract check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
