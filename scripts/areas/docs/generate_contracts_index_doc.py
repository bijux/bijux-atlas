#!/usr/bin/env python3
# Purpose: generate docs/_generated/contracts-index.md from CONTRACT.md and schema files.
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
OUT = ROOT / "docs" / "_generated" / "contracts-index.md"
SCAN_ROOTS = [
    ROOT / "ops",
    ROOT / "configs",
    ROOT / "makefiles",
    ROOT / "docker",
]


def rel(path: Path) -> str:
    return path.relative_to(ROOT).as_posix()


def main() -> int:
    contracts: list[Path] = []
    schemas: list[Path] = []
    for scan_root in SCAN_ROOTS:
        contracts.extend(scan_root.glob("**/CONTRACT.md"))
        schemas.extend(scan_root.glob("**/*.schema.json"))
    contracts = sorted(contracts)
    schemas = sorted(schemas)
    lines = [
        "# Contracts And Schemas Index (Generated)",
        "",
        "Generated from repository files. Do not edit manually.",
        "",
        f"- CONTRACT files: `{len(contracts)}`",
        f"- Schema files: `{len(schemas)}`",
        "",
        "## CONTRACT.md Files",
        "",
    ]
    lines.extend(f"- `{rel(path)}`" for path in contracts)
    lines += ["", "## Schema Files", ""]
    lines.extend(f"- `{rel(path)}`" for path in schemas)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(OUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
