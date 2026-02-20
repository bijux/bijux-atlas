#!/usr/bin/env python3
# Purpose: fail when :latest image tags appear in docker/k8s/scripts content.
# Inputs: docker, ops, scripts, and makefiles content.
# Outputs: non-zero when latest tags are found.
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[3]
needle = re.compile(r":[Ll][Aa][Tt][Ee][Ss][Tt](\b|@)")
scan_files = [
    ROOT / "docker/Dockerfile",
    *sorted((ROOT / "ops").rglob("*.yaml")),
    *sorted((ROOT / "ops").rglob("*.yml")),
    *sorted((ROOT / "ops").rglob("*.sh")),
    *sorted((ROOT / "scripts").rglob("*.sh")),
    *sorted((ROOT / "makefiles").rglob("*.mk")),
]
errors: list[str] = []
for path in scan_files:
    if not path.exists():
        continue
    text = path.read_text(encoding="utf-8", errors="ignore")
    for i, line in enumerate(text.splitlines(), start=1):
        if "releases/latest/download" in line:
            continue
        line_l = line.lower()
        relevant = (
            "image:" in line_l
            or line_l.strip().startswith("from ")
            or "docker run" in line_l
            or "docker pull" in line_l
            or "--image=" in line_l
            or "helm upgrade" in line_l
            or "helm install" in line_l
        )
        if relevant and needle.search(line):
            errors.append(f"{path.relative_to(ROOT)}:{i}: {line.strip()}")

if errors:
    print("latest-tag policy check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("no latest tags policy passed")
