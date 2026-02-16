#!/usr/bin/env python3
import json
import os
from pathlib import Path

root = Path(__file__).resolve().parents[2]
results = Path(os.environ.get("PERF_RESULTS_DIR", root / "artifacts/perf/results"))
out_dir = Path(os.environ.get("PERF_OUT_DIR", root / "artifacts/perf"))
out_dir.mkdir(parents=True, exist_ok=True)

rows = []
for file in sorted(results.glob("*.summary.json")):
    data = json.loads(file.read_text())
    m = data.get("metrics", {})
    dur = m.get("http_req_duration", {}).get("values", {})
    fail = m.get("http_req_failed", {}).get("values", {})
    rows.append({
        "suite": file.stem.replace(".summary", ""),
        "p50_ms": dur.get("p(50)", 0.0),
        "p95_ms": dur.get("p(95)", 0.0),
        "p99_ms": dur.get("p(99)", 0.0),
        "fail_rate": fail.get("rate", 0.0),
    })

baseline = {
    "version": os.environ.get("GITHUB_SHA", "local"),
    "rows": rows,
    "runtime": {},
}

stats_file = out_dir / "docker_stats.json"
if stats_file.exists():
    raw = stats_file.read_text().strip().splitlines()
    if raw:
        # one JSON object per line from docker stats --format '{{json .}}'
        first = json.loads(raw[0])
        baseline["runtime"] = {
            "cpu_percent": first.get("CPUPerc", ""),
            "memory_usage": first.get("MemUsage", ""),
        }
(out_dir / "baseline.json").write_text(json.dumps(baseline, indent=2) + "\n")

lines = [
    "# Performance Report",
    "",
    f"Version: `{baseline['version']}`",
    "",
    "| Suite | p50 (ms) | p95 (ms) | p99 (ms) | fail rate |",
    "|---|---:|---:|---:|---:|",
]
for r in rows:
    lines.append(
        f"| {r['suite']} | {r['p50_ms']:.2f} | {r['p95_ms']:.2f} | {r['p99_ms']:.2f} | {r['fail_rate']:.4f} |"
    )
if baseline["runtime"]:
    lines.extend(
        [
            "",
            "## Runtime Snapshot",
            "",
            f"- CPU: `{baseline['runtime'].get('cpu_percent', '')}`",
            f"- Memory: `{baseline['runtime'].get('memory_usage', '')}`",
        ]
    )

(out_dir / "report.md").write_text("\n".join(lines) + "\n")
print(f"wrote {out_dir / 'report.md'} and {out_dir / 'baseline.json'}")
