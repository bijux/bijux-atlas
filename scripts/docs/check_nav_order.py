#!/usr/bin/env python3
# Purpose: enforce top-level mkdocs nav ordering policy.
# Inputs: mkdocs.yml
# Outputs: non-zero exit on order mismatch.
from __future__ import annotations

from pathlib import Path
import re

ROOT = Path(__file__).resolve().parents[2]
mkdocs = (ROOT / "mkdocs.yml").read_text(encoding="utf-8")
expected = [
    "Product",
    "Quickstart",
    "Reference",
    "Contracts",
    "Operations",
    "Development",
    "Architecture",
    "ADRs",
]
nav_start = mkdocs.find("\nnav:\n")
if nav_start == -1:
    print("nav ordering check failed: missing nav section")
    raise SystemExit(1)
nav_text = mkdocs[nav_start:]
found = re.findall(r"^  - ([A-Za-z]+):\s*$", nav_text, flags=re.M)
if found[: len(expected)] != expected:
    print("nav ordering check failed")
    print(f"expected: {expected}")
    print(f"found:    {found[:len(expected)]}")
    raise SystemExit(1)
print("nav ordering check passed")
