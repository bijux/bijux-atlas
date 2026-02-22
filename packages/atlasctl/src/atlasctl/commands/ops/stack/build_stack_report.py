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

from atlasctl.core.runtime.paths import write_text_file


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

    root = Path(__file__).resolve().parents[5]
    out_dir = root / args.out_dir

    values_src = root / args.values_file
    values_dst = out_dir / "helm-values-used.yaml"
    if values_src.exists():
        write_text_file(values_dst, values_src.read_text(encoding="utf-8", errors="replace"), encoding="utf-8")
    else:
        write_text_file(values_dst, "# missing values file\n", encoding="utf-8")

    k6_candidates = sorted((root / "artifacts/perf/results").glob("*.summary.json"))
    k6_dst = out_dir / "k6-summary.json"
    if k6_candidates:
        merged = {"summaries": []}
        for p in k6_candidates:
            merged["summaries"].append({"file": p.name, "summary": json.loads(p.read_text())})
        write_text_file(k6_dst, json.dumps(merged, indent=2) + "\n", encoding="utf-8")
    else:
        write_text_file(k6_dst, json.dumps({"summaries": []}, indent=2) + "\n", encoding="utf-8")

    metrics_src = root / "artifacts/ops/obs/metrics.prom"
    metrics_dst = out_dir / "metrics.prom"
    write_text_file(
        metrics_dst,
        metrics_src.read_text(encoding="utf-8", errors="replace") if metrics_src.exists() else "",
        encoding="utf-8",
    )

    trace_src = root / "artifacts/ops/obs/traces.snapshot.log"
    trace_dst = out_dir / "traces.snapshot.log"
    write_text_file(
        trace_dst,
        trace_src.read_text(encoding="utf-8", errors="replace") if trace_src.exists() else "",
        encoding="utf-8",
    )

    dashboard_src = root / "ops/obs/grafana/atlas-observability-dashboard.json"
    dashboard_txt = out_dir / "dashboard-screenshot.txt"
    if dashboard_src.exists():
        try:
            dash = json.loads(dashboard_src.read_text())
            lines = [
                "# Dashboard Screenshot (Text Export)",
                f"title: {dash.get('title', '')}",
                "",
                "panels:",
            ]
            for panel in dash.get("panels", []):
                title = panel.get("title", "<untitled>")
                ptype = panel.get("type", "unknown")
                lines.append(f"- [{ptype}] {title}")
            write_text_file(dashboard_txt, "\n".join(lines) + "\n", encoding="utf-8")
        except Exception:
            write_text_file(dashboard_txt, "# dashboard export failed\n", encoding="utf-8")
    else:
        write_text_file(dashboard_txt, "# missing dashboard json\n", encoding="utf-8")

    logs_dst = out_dir / "logs-excerpt.log"
    logs_candidates = sorted((root / "artifacts/ops").glob("*/logs/*.log"))
    if logs_candidates:
        lines = logs_candidates[-1].read_text(errors="replace").splitlines()[-200:]
        write_text_file(logs_dst, "\n".join(lines) + ("\n" if lines else ""), encoding="utf-8")
    else:
        write_text_file(logs_dst, "", encoding="utf-8")

    rendered_src = out_dir / "rendered-manifests.yaml"
    rendered_dst = out_dir / "rendered-manifests.yaml"
    if rendered_src.exists():
        write_text_file(rendered_dst, rendered_src.read_text(encoding="utf-8", errors="replace"), encoding="utf-8")
    else:
        write_text_file(rendered_dst, "# missing rendered manifests\n", encoding="utf-8")

    stack_hash_inputs = [
        root / "configs/ops/tool-versions.json",
        root / "ops/load/suites/suites.json",
        root / "configs/perf/k6-thresholds.v1.json",
        values_src,
    ]
    stack_version_hash = sha256_files(stack_hash_inputs)
    write_text_file(out_dir / "stack-version-hash.txt", stack_version_hash + "\n", encoding="utf-8")

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
            "dashboard_screenshot": dashboard_txt.relative_to(root).as_posix(),
            "logs_excerpt": logs_dst.relative_to(root).as_posix(),
            "rendered_manifests": rendered_dst.relative_to(root).as_posix(),
            "pass_fail_summary": "artifacts/stack-report/pass-fail-summary.json",
        },
    }
    write_text_file(out_dir / "pass-fail-summary.json", json.dumps(summary, indent=2) + "\n", encoding="utf-8")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
