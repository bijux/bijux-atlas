#!/usr/bin/env python3
# Purpose: generate scripts/README.md index of scripts and owners.
# Inputs: scripts tree.
# Outputs: scripts/README.md.
from pathlib import Path

ROOT = Path(__file__).resolve().parents[1]
SCRIPTS = ROOT / "scripts"
OUT = SCRIPTS / "README.md"

owner_map = {
    "contracts": "contracts",
    "docs": "docs-governance",
    "layout": "repo-surface",
    "observability": "operations",
    "perf": "performance-compat",
    "_internal": "internal",
    "release": "release-engineering",
    "fixtures": "dataset-ops",
    "bootstrap": "developer-experience",
}

files = sorted(
    p for p in SCRIPTS.rglob("*") if p.is_file() and p.name != "README.md"
)

lines = [
    "# Scripts Index",
    "",
    "Generated file. Do not edit manually.",
    "",
    "| Script | Owner |",
    "|---|---|",
]

for p in files:
    rel = p.relative_to(ROOT).as_posix()
    top = rel.split("/")[1] if "/" in rel else ""
    owner = owner_map.get(top, "platform")
    lines.append(f"| `{rel}` | `{owner}` |")

OUT.write_text("\n".join(lines) + "\n")
print(OUT)
