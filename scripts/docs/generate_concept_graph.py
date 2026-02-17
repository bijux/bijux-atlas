#!/usr/bin/env python3
from __future__ import annotations

from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[2]
REGISTRY = ROOT / "docs" / "_style" / "concepts.yml"
OUT = ROOT / "docs" / "_generated" / "concepts.md"


def main() -> int:
    data = yaml.safe_load(REGISTRY.read_text(encoding="utf-8"))
    concepts = data.get("concepts", []) if isinstance(data, dict) else []

    lines = [
        "# Concept Graph",
        "",
        "- Owner: `docs-governance`",
        "",
        "## What",
        "",
        "Generated mapping of concept IDs to canonical and pointer pages.",
        "",
        "## Why",
        "",
        "Provides a deterministic lookup for concept ownership.",
        "",
        "## Scope",
        "",
        "Concept registry entries from `docs/_style/concepts.yml`.",
        "",
        "## Non-goals",
        "",
        "No semantic interpretation beyond declared links.",
        "",
        "## Contracts",
        "",
        "- Exactly one canonical page per concept.",
        "- Pointer pages must reference canonical page.",
        "",
        "## Failure modes",
        "",
        "Registry drift causes stale concept ownership.",
        "",
        "## How to verify",
        "",
        "```bash",
        "$ python3 scripts/docs/check_concept_registry.py",
        "$ make docs",
        "```",
        "",
        "Expected output: concept checks pass.",
        "",
        "## Concepts",
        "",
    ]

    for c in concepts:
        cid = c["id"]
        canonical = c["canonical"].replace("docs/", "")
        pointers = [p.replace("docs/", "") for p in c.get("pointers", [])]
        lines.append(f"### `{cid}`")
        lines.append("")
        lines.append(f"- Canonical: [{canonical}](../{canonical})")
        if pointers:
            for p in pointers:
                lines.append(f"- Pointer: [{p}](../{p})")
        else:
            lines.append("- Pointer: none")
        lines.append("")

    lines.extend(
        [
            "## See also",
            "",
            "- [Concept Registry](../_style/CONCEPT_REGISTRY.md)",
            "- [Concept IDs](../_style/concept-ids.md)",
            "- [Docs Home](../index.md)",
            "",
        ]
    )

    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines), encoding="utf-8")
    print(f"generated {OUT}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
