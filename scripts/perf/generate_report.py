#!/usr/bin/env python3
import json
import os
import re
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
    "cold_start": {},
    "soak_memory": {},
}

cold_start_file = out_dir / "cold-start" / "result.json"
if cold_start_file.exists():
    baseline["cold_start"] = json.loads(cold_start_file.read_text())


def parse_mem_bytes(raw: str) -> int:
    if not raw:
        return 0
    used = raw.split("/")[0].strip()
    m = re.match(r"^([0-9]*\.?[0-9]+)([KMGTP]i?)?B?$", used)
    if not m:
        return 0
    value = float(m.group(1))
    unit = (m.group(2) or "").lower()
    mult = {
        "": 1,
        "k": 1000,
        "m": 1000**2,
        "g": 1000**3,
        "t": 1000**4,
        "ki": 1024,
        "mi": 1024**2,
        "gi": 1024**3,
        "ti": 1024**4,
    }.get(unit, 1)
    return int(value * mult)


soak_start = out_dir / "docker_stats_soak_start.json"
soak_end = out_dir / "docker_stats_soak_end.json"
if soak_start.exists() and soak_end.exists():
    start_lines = soak_start.read_text().strip().splitlines()
    end_lines = soak_end.read_text().strip().splitlines()
    if start_lines and end_lines:
        start_obj = json.loads(start_lines[0])
        end_obj = json.loads(end_lines[0])
        start_mem = parse_mem_bytes(start_obj.get("MemUsage", ""))
        end_mem = parse_mem_bytes(end_obj.get("MemUsage", ""))
        baseline["soak_memory"] = {
            "start_bytes": start_mem,
            "end_bytes": end_mem,
            "growth_bytes": max(0, end_mem - start_mem),
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
if baseline["cold_start"]:
    lines.extend(
        [
            "",
            "## Cold Start",
            "",
            f"- first request http_code: `{baseline['cold_start'].get('http_code', 0)}`",
            f"- first request latency (ms): `{baseline['cold_start'].get('cold_start_ms', 0)}`",
        ]
    )
if baseline["soak_memory"]:
    lines.extend(
        [
            "",
            "## Soak Memory",
            "",
            f"- start bytes: `{baseline['soak_memory'].get('start_bytes', 0)}`",
            f"- end bytes: `{baseline['soak_memory'].get('end_bytes', 0)}`",
            f"- growth bytes: `{baseline['soak_memory'].get('growth_bytes', 0)}`",
        ]
    )

(out_dir / "report.md").write_text("\n".join(lines) + "\n")
print(f"wrote {out_dir / 'report.md'} and {out_dir / 'baseline.json'}")
