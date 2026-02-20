#!/usr/bin/env python3
from __future__ import annotations

import argparse
import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[4]


def _exists(path: Path) -> bool:
    return path.exists() and path.stat().st_size >= 0


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--suite", required=True)
    parser.add_argument("--status", required=True)
    parser.add_argument("--out-dir", required=True)
    args = parser.parse_args()

    out_dir = Path(args.out_dir)
    out_dir.mkdir(parents=True, exist_ok=True)

    metrics = ROOT / "artifacts/ops/obs/metrics.prom"
    traces = ROOT / "artifacts/ops/obs/traces.snapshot.log"
    exemplars = ROOT / "artifacts/ops/obs/traces.exemplars.log"
    endpoint_contract = ROOT / "ops/obs/contract/endpoint-observability-contract.json"
    obs_budgets = ROOT / "configs/ops/obs/budgets.json"

    payload = {
        "run_id": args.run_id,
        "suite": args.suite,
        "status": args.status,
        "contracts": {
            "endpoint_observability_contract": str(endpoint_contract.relative_to(ROOT)),
            "obs_budgets": str(obs_budgets.relative_to(ROOT)),
        },
        "evidence": {
            "metrics_snapshot_present": _exists(metrics),
            "trace_snapshot_present": _exists(traces),
            "trace_exemplars_present": _exists(exemplars),
        },
    }
    (out_dir / "obs-conformance.json").write_text(
        json.dumps(payload, indent=2, sort_keys=True) + "\n",
        encoding="utf-8",
    )

    lines = [
        "# Observability Conformance",
        "",
        f"- Run ID: `{args.run_id}`",
        f"- Suite: `{args.suite}`",
        f"- Status: `{args.status}`",
        "",
        "## Contract Inputs",
        f"- `{endpoint_contract.relative_to(ROOT)}`",
        f"- `{obs_budgets.relative_to(ROOT)}`",
        "",
        "## Evidence Coverage",
        f"- Metrics snapshot: {'yes' if payload['evidence']['metrics_snapshot_present'] else 'no'}",
        f"- Trace snapshot: {'yes' if payload['evidence']['trace_snapshot_present'] else 'no'}",
        f"- Trace exemplars: {'yes' if payload['evidence']['trace_exemplars_present'] else 'no'}",
        "",
    ]
    (out_dir / "obs-conformance.md").write_text("\n".join(lines), encoding="utf-8")
    print((out_dir / "obs-conformance.json").as_posix())
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
