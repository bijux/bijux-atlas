#!/usr/bin/env python3
# Purpose: generate docs/_generated/config-keys.md from configs/config-key-registry.md.
from __future__ import annotations

from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
REGISTRY = ROOT / "configs" / "config-key-registry.md"
OUT = ROOT / "docs" / "_generated" / "config-keys.md"


def main() -> int:
    keys: list[str] = []
    for line in REGISTRY.read_text(encoding="utf-8").splitlines():
        line = line.strip()
        if not line.startswith("- `") or not line.endswith("`"):
            continue
        keys.append(line[3:-1])

    lines = [
        "# Config Keys (Generated)",
        "",
        "Generated from `configs/config-key-registry.md`. Do not edit manually.",
        "",
        f"- Count: `{len(keys)}`",
        "",
        "## Keys",
        "",
    ]
    lines.extend(f"- `{k}`" for k in keys)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(OUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
