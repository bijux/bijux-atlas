#!/usr/bin/env python3
from __future__ import annotations

import json
import re
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
K8S_DIR = ROOT / "docs" / "operations" / "k8s"
keys = set(json.loads((ROOT / "docs" / "contracts" / "CHART_VALUES.json").read_text())["top_level_keys"])

errors: list[str] = []
for path in sorted(K8S_DIR.glob("*.md")):
    if path.name == "INDEX.md":
        continue
    text = path.read_text(encoding="utf-8")
    refs = re.findall(r"`values\.([a-z][a-zA-Z0-9_\-]*)`", text)
    refs = [ref for ref in refs if ref != "yaml"]
    if not refs:
        errors.append(f"{path}: missing values.<key> references")
        continue
    for ref in refs:
        if ref not in keys:
            errors.append(f"{path}: unknown chart values key `{ref}`")

if errors:
    print("k8s docs contract check failed:", file=sys.stderr)
    for e in errors:
        print(f"- {e}", file=sys.stderr)
    sys.exit(1)

print("k8s docs contract check passed")
