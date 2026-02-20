#!/usr/bin/env python3
# Purpose: generate canonical make target inventory JSON and markdown from SSOT+ownership.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
SSOT = ROOT / "configs" / "make" / "public-targets.json"
OWNERS = ROOT / "makefiles" / "ownership.json"
OUT_JSON = ROOT / "makefiles" / "targets.json"
OUT_MD = ROOT / "docs" / "_generated" / "make-targets.md"


def main() -> int:
    ssot = json.loads(SSOT.read_text(encoding="utf-8"))
    owners = json.loads(OWNERS.read_text(encoding="utf-8"))
    entries = []
    for item in sorted(ssot.get("public_targets", []), key=lambda e: e["name"]):
        name = item["name"]
        meta = owners.get(name, {})
        lanes = item.get("lanes", [])
        entries.append(
            {
                "name": name,
                "description": item.get("description", ""),
                "owner": meta.get("owner", ""),
                "area": item.get("area", meta.get("area", "")),
                "lane": lanes[0] if lanes else "",
                "lanes": lanes,
            }
        )

    payload = {
        "schema_version": 1,
        "source": {
            "ssot": str(SSOT.relative_to(ROOT)),
            "ownership": str(OWNERS.relative_to(ROOT)),
        },
        "targets": entries,
    }

    OUT_JSON.parent.mkdir(parents=True, exist_ok=True)
    OUT_JSON.write_text(json.dumps(payload, indent=2, sort_keys=True) + "\n", encoding="utf-8")

    lines = [
        "# Make Targets",
        "",
        "Generated from `makefiles/targets.json`. Do not edit manually.",
        "",
        "| target | description | owner | area | lane |",
        "|---|---|---|---|---|",
    ]
    for t in entries:
        desc = t["description"].replace("|", "/")
        lines.append(f"| `{t['name']}` | {desc} | `{t['owner']}` | `{t['area']}` | `{t['lane']}` |")

    OUT_MD.parent.mkdir(parents=True, exist_ok=True)
    OUT_MD.write_text("\n".join(lines) + "\n", encoding="utf-8")

    print(OUT_JSON.relative_to(ROOT))
    print(OUT_MD.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
