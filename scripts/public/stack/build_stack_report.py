#!/usr/bin/env python3
# owner: platform
# purpose: build deterministic stack-full report bundle under artifacts/stack-report.
# stability: public
# called-by: make stack-full
# Purpose: materialize stack-full artifact bundle and pass/fail summary in a fixed layout.
# Inputs: --status, --run-id, --out-dir, and optional --values-file path.
# Outputs: writes artifacts/stack-report/* including pass-fail-summary.json and stack-version-hash.txt.

from __future__ import annotations

import argparse
import datetime as dt
import hashlib
import json
from pathlib import Path


def sha256_files(paths: list[Path]) -> str:
    h = hashlib.sha256()
    for path in sorted(paths):
        h.update(path.as_posix().encode("utf-8"))
        if path.exists() and path.is_file():
            h.update(path.read_bytes())
    return h.hexdigest()


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--status", required=True, choices=["passed", "failed"])
    parser.add_argument("--run-id", required=True)
    parser.add_argument("--out-dir", default="artifacts/stack-report")
    parser.add_argument("--values-file", default="ops/k8s/values/local.yaml")
    args = parser.parse_args()

    root = Path(__file__).resolve().parents[3]
    out_dir = root / args.out_dir
    out_dir.mkdir(parents=True, exist_ok=True)

    values_src = root / args.values_file
    values_dst = out_dir / "helm-values-used.yaml"
    if values_src.exists():
        values_dst.write_bytes(values_src.read_bytes())
    else:
        values_dst.write_text("# missing values file\n")

    k6_candidates = sorted((root / "artifacts/perf/results").glob("*.summary.json"))
    k6_dst = out_dir / "k6-summary.json"
    if k6_candidates:
        merged = {"summaries": []}
        for p in k6_candidates:
            merged["summaries"].append({"file": p.name, "summary": json.loads(p.read_text())})
        k6_dst.write_text(json.dumps(merged, indent=2) + "\n")
    else:
        k6_dst.write_text(json.dumps({"summaries": []}, indent=2) + "\n")

    metrics_src = root / "artifacts/ops/observability/metrics.prom"
    metrics_dst = out_dir / "metrics.prom"
    metrics_dst.write_bytes(metrics_src.read_bytes() if metrics_src.exists() else b"")

    trace_src = root / "artifacts/ops/observability/traces.snapshot.log"
    trace_dst = out_dir / "traces.snapshot.log"
    trace_dst.write_bytes(trace_src.read_bytes() if trace_src.exists() else b"")

    logs_dst = out_dir / "logs-excerpt.log"
    logs_candidates = sorted((root / "artifacts/ops").glob("*/logs/*.log"))
    if logs_candidates:
        lines = logs_candidates[-1].read_text(errors="replace").splitlines()[-200:]
        logs_dst.write_text("\n".join(lines) + ("\n" if lines else ""))
    else:
        logs_dst.write_text("")

    rendered_src = out_dir / "rendered-manifests.yaml"
    rendered_dst = out_dir / "rendered-manifests.yaml"
    if rendered_src.exists():
        rendered_dst.write_bytes(rendered_src.read_bytes())
    else:
        rendered_dst.write_text("# missing rendered manifests\n")

    stack_hash_inputs = [
        root / "ops/tool-versions.json",
        root / "ops/load/suites/suites.json",
        root / "configs/perf/thresholds.json",
        values_src,
    ]
    stack_version_hash = sha256_files(stack_hash_inputs)
    (out_dir / "stack-version-hash.txt").write_text(stack_version_hash + "\n")

    summary = {
        "schema_version": 1,
        "stack_version_hash": stack_version_hash,
        "status": args.status,
        "run_id": args.run_id,
        "generated_at_utc": dt.datetime.now(dt.timezone.utc).strftime("%Y-%m-%dT%H:%M:%SZ"),
        "artifacts": {
            "helm_values_used": values_dst.relative_to(root).as_posix(),
            "k6_summary": k6_dst.relative_to(root).as_posix(),
            "metrics_snapshot": metrics_dst.relative_to(root).as_posix(),
            "trace_snapshot": trace_dst.relative_to(root).as_posix(),
            "logs_excerpt": logs_dst.relative_to(root).as_posix(),
            "rendered_manifests": rendered_dst.relative_to(root).as_posix(),
            "pass_fail_summary": "artifacts/stack-report/pass-fail-summary.json",
        },
    }
    (out_dir / "pass-fail-summary.json").write_text(json.dumps(summary, indent=2) + "\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
