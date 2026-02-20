#!/usr/bin/env python3
# Purpose: enforce canonical chart path usage via ops/k8s/charts/bijux-atlas.
# Inputs: repository text files.
# Outputs: non-zero on non-canonical chart path references.
from pathlib import Path
import re
import sys

root = Path(__file__).resolve().parents[2]
exempt = {
    "docs/development/symlinks.md",
    "docs/development/root-inventory.md",
    "packages/atlasctl/src/atlasctl/layout_checks/check_chart_canonical_path.sh",
}

violations = []
for path in root.rglob("*"):
    if not path.is_file():
        continue
    rel = path.relative_to(root).as_posix()
    if rel.startswith((".git/", "ops/k8s/charts/", "charts/", "artifacts/")):
        continue
    if rel in exempt:
        continue

    try:
        text = path.read_text()
    except Exception:
        continue

    for m in re.finditer(r"charts/bijux-atlas", text):
        prefix = text[max(0, m.start() - 8):m.start()]
        if prefix.endswith("ops/k8s/"):
            continue
        line = text.count("\n", 0, m.start()) + 1
        violations.append(f"{rel}:{line}:{m.group(0)}")

if violations:
    print("non-canonical chart path references found:", file=sys.stderr)
    for v in violations:
        print(f"- {v}", file=sys.stderr)
    raise SystemExit(1)

print("chart canonical path check passed")
