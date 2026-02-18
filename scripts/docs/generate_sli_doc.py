#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
src = ROOT / "configs/ops/slo/slis.v1.json"
out = ROOT / "docs/operations/slo/SLIS.md"

payload = json.loads(src.read_text(encoding="utf-8"))
slis = payload.get("slis", [])

lines: list[str] = []
lines.append("# SLIs (v1)")
lines.append("")
lines.append("- Generated from `configs/ops/slo/slis.v1.json`.")
lines.append("")
lines.append("| SLI | Goal | Primary Metric | Secondary Metric | Status |")
lines.append("|---|---|---|---|---|")
for sli in slis:
    lines.append(
        "| {name} | {goal} | `{metric}` | {secondary} | `{status}` |".format(
            name=sli.get("name", ""),
            goal=sli.get("goal", ""),
            metric=sli.get("metric", ""),
            secondary=f"`{sli.get('secondary_metric')}`" if sli.get("secondary_metric") else "-",
            status=sli.get("status", "unknown"),
        )
    )

lines.append("")
lines.append("## Endpoint Class Mapping")
lines.append("")
lines.append("- `cheap`: `^/health$`, `^/version$`, `^/metrics$`")
lines.append("- `standard`: `^/v1/genes$`, `^/v1/genes/[^/]+$`")
lines.append("- `heavy`: `^/v1/genes/[^/]+/(diff|region|sequence)$`")

out.write_text("\n".join(lines) + "\n", encoding="utf-8")
print(f"generated {out}")
