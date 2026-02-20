#!/usr/bin/env python3
# Purpose: generate docs/architecture/architecture-map.md from workspace crate metadata.
# Inputs: cargo metadata for workspace crates.
# Outputs: docs/architecture/architecture-map.md (deterministic).
import json
import subprocess
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "docs" / "architecture" / "architecture-map.md"

CATEGORY_HINTS = {
    "bijux-atlas-api": "api-surface",
    "bijux-atlas-server": "runtime-server",
    "bijux-atlas-query": "query-engine",
    "bijux-atlas-store": "artifact-store",
    "bijux-atlas-ingest": "ingest-pipeline",
    "bijux-atlas-cli": "cli-ops",
    "bijux-atlas-model": "shared-model",
    "bijux-atlas-core": "shared-core",
    "bijux-atlas-policies": "policy-contracts",
}


def cargo_metadata() -> dict:
    out = subprocess.check_output(
        ["cargo", "metadata", "--locked", "--format-version", "1", "--no-deps"],
        cwd=ROOT,
        text=True,
    )
    return json.loads(out)


def main() -> int:
    meta = cargo_metadata()
    packages = {
        p["name"]: p
        for p in meta["packages"]
        if p["name"].startswith("bijux-atlas-")
    }
    names = sorted(packages.keys())
    lines = [
        "# Architecture Map",
        "",
        "- Owner: `atlas-platform`",
        "- Stability: `stable`",
        "",
        "Generated crate-level architecture map from workspace metadata.",
        "",
        "## Crate Nodes",
        "",
        "| Crate | Role | Internal Dependencies |",
        "| --- | --- | --- |",
    ]
    for name in names:
        pkg = packages[name]
        deps = sorted(
            d["name"]
            for d in pkg.get("dependencies", [])
            if d["name"].startswith("bijux-atlas-")
        )
        dep_str = ", ".join(f"`{d}`" for d in deps) if deps else "`(none)`"
        role = CATEGORY_HINTS.get(name, "unspecified")
        lines.append(f"| `{name}` | `{role}` | {dep_str} |")

    lines += [
        "",
        "## Runtime Direction",
        "",
        "`bijux-atlas-server -> bijux-atlas-query -> bijux-atlas-store -> immutable artifacts`",
        "",
        "## Notes",
        "",
        "- This file is generated; do not hand-edit.",
        "- Regenerate via `python3 scripts/areas/docs/generate_architecture_map.py`.",
    ]
    OUT.write_text("\n".join(lines) + "\n")
    print(f"wrote {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
