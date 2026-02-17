#!/usr/bin/env python3
# Purpose: generate scripts/README.md index of scripts and owners.
# Inputs: scripts tree.
# Outputs: scripts/README.md.
from pathlib import Path
import fnmatch
import re

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
    "| Script | Owner | Stability |",
    "|---|---|---|",
]

entrypoints = (SCRIPTS / "ENTRYPOINTS.md").read_text() if (SCRIPTS / "ENTRYPOINTS.md").exists() else ""
stability_map = {"public": [], "internal": [], "private": []}
section = None
for line in entrypoints.splitlines():
    m = re.match(r"^##\s+(Public|Internal|Private)\s*$", line.strip())
    if m:
        section = m.group(1).lower()
        continue
    if section and line.strip().startswith("- `"):
        stability_map[section].append(line.strip()[3:-1].split(" ")[0])

for p in files:
    rel = p.relative_to(ROOT).as_posix()
    top = rel.split("/")[1] if "/" in rel else ""
    owner = owner_map.get(top, "platform")
    stability = "internal"
    for level, patterns in stability_map.items():
        if any(fnmatch.fnmatch(rel, pat) for pat in patterns):
            stability = level
            break
    lines.append(f"| `{rel}` | `{owner}` | `{stability}` |")

OUT.write_text("\n".join(lines) + "\n")
print(OUT)
