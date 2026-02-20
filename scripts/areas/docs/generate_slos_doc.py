#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
src = ROOT / "configs/ops/slo/slo.v1.json"
out = ROOT / "docs/operations/slo/SLOS.md"

payload = json.loads(src.read_text(encoding="utf-8"))
slis = {item["id"]: item for item in payload.get("slis", [])}

lines: list[str] = []
lines.append("# SLO Targets (v1)")
lines.append("")
lines.append("- Generated from `configs/ops/slo/slo.v1.json`.")
lines.append("")
lines.append("| SLO ID | SLI | Target | Window | Threshold |")
lines.append("|---|---|---:|---|---|")
for slo in payload.get("slos", []):
    sli_id = slo.get("sli", "")
    sli_name = slis.get(sli_id, {}).get("name", sli_id)
    threshold = "-"
    if isinstance(slo.get("threshold"), dict):
        th = slo["threshold"]
        threshold = f"`{th.get('operator')} {th.get('value')} {th.get('unit')}`"
    lines.append(
        f"| `{slo.get('id','')}` | `{sli_name}` | `{slo.get('target','')}` | `{slo.get('window','')}` | {threshold} |"
    )

lines.append("")
lines.append("## v1 Pragmatic Targets")
lines.append("")
lines.append("- `/readyz` availability: `99.9%` over `30d`.")
lines.append("- Success: cheap `99.99%`, standard `99.9%`, heavy `99.0%` over `30d`.")
lines.append("- Latency p99 thresholds: cheap `< 50ms`, standard `< 300ms`, heavy `< 2s`.")
lines.append("- Overload cheap survival: `> 99.99%`.")
lines.append("- Shed policy: heavy shedding tolerated; standard shedding bounded.")
lines.append("- Registry freshness: refresh age `< 10m` for `99%` of windows.")
lines.append("- Store objective: p95 latency bounded and error rate `< 0.5%`.")

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"generated {out}")
