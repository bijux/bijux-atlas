#!/usr/bin/env python3
# Purpose: generate script indexes for scripts governance.
# Inputs: scripts tree, scripts/ENTRYPOINTS.md, Makefile + makefiles/*.mk.
# Outputs: scripts/README.md, scripts/INDEX.md, docs/_generated/scripts-surface.md.
from pathlib import Path
import fnmatch
import re

def _repo_root() -> Path:
    cur = Path(__file__).resolve()
    for parent in [cur.parent, *cur.parents]:
        if (parent / "Makefile").exists() and (parent / "scripts").is_dir():
            return parent
    raise SystemExit("could not locate repository root from script path")


ROOT = _repo_root()
SCRIPTS = ROOT / "scripts"
OUT_README = SCRIPTS / "README.md"
OUT_INDEX = SCRIPTS / "INDEX.md"
OUT_SURFACE = ROOT / "docs/_generated/scripts-surface.md"

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
    p
    for p in SCRIPTS.rglob("*")
    if p.is_file() and p.name not in {"README.md", "INDEX.md"}
)

mk_files = [ROOT / "Makefile"] + sorted((ROOT / "makefiles").glob("*.mk"))
mk_text = "\n".join(p.read_text(encoding="utf-8") for p in mk_files)
called_by: dict[str, list[str]] = {}
for mk in mk_files:
    txt = mk.read_text(encoding="utf-8")
    for m in re.finditer(r"^\s*([a-zA-Z0-9_.-]+):", txt, re.MULTILINE):
        target = m.group(1)
        segment = txt[m.start() : txt.find("\n\n", m.start()) if txt.find("\n\n", m.start()) != -1 else len(txt)]
        refs = set(re.findall(r"\./(scripts/[^\s\"\\;]+)", segment))
        refs.update(re.findall(r"(?:python3|python)\s+(scripts/[^\s\"\\;]+)", segment))
        for ref in refs:
            called_by.setdefault(ref, []).append(target)

lines = [
    "# Scripts",
    "",
    "Categories:",
    "- `scripts/check/`: validators and lint gates",
    "- `scripts/gen/`: generators for docs and inventories",
    "- `scripts/ci/`: CI-only glue",
    "- `scripts/dev/`: local helpers",
    "- `scripts/lib/`: shared script libraries",
    "- `scripts/python/`: reusable Python modules",
    "- `scripts/bin/`: thin entrypoints only",
    "",
    "Policy: scripts are internal unless listed in `configs/ops/public-surface.json` or the `public` section in `scripts/ENTRYPOINTS.md`.",
    "",
    "## Full Inventory",
    "",
    "# Scripts Index",
    "",
    "Generated file. Do not edit manually.",
    "",
    "| Script | Owner | Stability | Called By |",
    "|---|---|---|---|",
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
    called = sorted(called_by.get(rel, []))
    called_txt = ", ".join(f"`{t}`" for t in called) if called else "-"
    lines.append(f"| `{rel}` | `{owner}` | `{stability}` | {called_txt} |")

content = "\n".join(lines) + "\n"
OUT_README.write_text(content, encoding="utf-8")
OUT_INDEX.write_text(content, encoding="utf-8")
surface_lines = [
    "# Scripts Surface",
    "",
    "Generated file. Do not edit manually.",
    "",
    "## Public scripts/bin entrypoints",
    "",
]
for p in sorted((SCRIPTS / "bin").glob("*")):
    if p.is_file():
        surface_lines.append(f"- `{p.relative_to(ROOT).as_posix()}`")
surface_lines.extend(["", "## Script Domains", ""])
for d in ["check", "gen", "ci", "dev", "lib", "python"]:
    surface_lines.append(f"- `scripts/{d}/`")
OUT_SURFACE.write_text("\n".join(surface_lines) + "\n", encoding="utf-8")
print(OUT_README)
print(OUT_INDEX)
print(OUT_SURFACE)
