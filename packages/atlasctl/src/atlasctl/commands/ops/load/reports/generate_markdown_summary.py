#!/usr/bin/env python3
# Purpose: generate markdown summary report from load result artifacts.
# Inputs: results directory with *.summary.json and *.meta.json
# Outputs: markdown report under artifacts/ops/<run-id>/load/reports/ and legacy summary path.
from __future__ import annotations

import json
import os
from pathlib import Path


ROOT = Path(__file__).resolve().parents[7]
RES = ROOT / "artifacts/perf/results"
RUN_OUT = ROOT / "artifacts/ops" / os.environ.get("OPS_RUN_ID", "manual") / "load" / "reports"
LEGACY_OUT = ROOT / "artifacts/ops/load/reports"
RUN_OUT.mkdir(parents=True, exist_ok=True)
LEGACY_OUT.mkdir(parents=True, exist_ok=True)


def bar(ms: float, max_ms: float = 3000.0) -> str:
    blocks = int(max(0.0, min(1.0, ms / max_ms)) * 10)
    return "#" * blocks + "-" * (10 - blocks)


def main() -> int:
    lines = [
        "# Load Summary Report",
        "",
        "| scenario | p95(ms) | p99(ms) | latency-shape | fail_rate | git_sha | image_digest | dataset_hash | dataset_release | policy_hash |",
        "|---|---:|---:|---|---:|---|---|---|---|---|",
    ]
    for f in sorted(RES.glob("*.summary.json")):
        d = json.loads(f.read_text(encoding="utf-8"))
        m = d.get("metrics", {})
        dur = m.get("http_req_duration", {}).get("values", {})
        fail = m.get("http_req_failed", {}).get("values", {})
        meta = f.with_suffix(".meta.json")
        meta_d = json.loads(meta.read_text(encoding="utf-8")) if meta.exists() else {}
        scenario = f.stem.replace(".summary", "")
        p95 = float(dur.get("p(95)", 0.0))
        p99 = float(dur.get("p(99)", 0.0))
        fail_rate = float(fail.get("rate", 0.0))
        lines.append(
            f"| {scenario} | {p95:.2f} | {p99:.2f} | {bar(p95)} | {fail_rate:.4f} | "
            f"{meta_d.get('git_sha','unknown')} | {meta_d.get('image_digest','unknown')} | "
            f"{meta_d.get('dataset_hash','unknown')} | {meta_d.get('dataset_release','unknown')} | "
            f"{meta_d.get('policy_hash','unknown')} |"
        )

    content = "\n".join(lines) + "\n"
    (RUN_OUT / "report.md").write_text(content, encoding="utf-8")
    (RUN_OUT / "summary.md").write_text(content, encoding="utf-8")
    (LEGACY_OUT / "summary.md").write_text(content, encoding="utf-8")
    print(RUN_OUT / "report.md")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
