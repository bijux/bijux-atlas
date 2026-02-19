#!/usr/bin/env python3
# Purpose: fail when :latest image tags appear in Dockerfile or k8s chart values/templates.
# Inputs: docker and ops/k8s yaml/dockerfile content.
# Outputs: non-zero when latest tags are found.
from __future__ import annotations

from pathlib import Path
import re
import sys

ROOT = Path(__file__).resolve().parents[2]
needle = re.compile(r":latest(\b|@)")
scan_files = [ROOT / "docker/Dockerfile", *sorted((ROOT / "ops/k8s").rglob("*.yaml")), *sorted((ROOT / "ops/k8s").rglob("*.yml"))]
errors: list[str] = []
for path in scan_files:
    text = path.read_text(encoding="utf-8", errors="ignore")
    for i, line in enumerate(text.splitlines(), start=1):
        if needle.search(line):
            errors.append(f"{path.relative_to(ROOT)}:{i}: {line.strip()}")

if errors:
    print("latest-tag policy check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    raise SystemExit(1)

print("no latest tags policy passed")
