from __future__ import annotations

import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
SRC = ROOT / "packages" / "atlasctl" / "src"
if str(SRC) not in sys.path:
    sys.path.insert(0, str(SRC))

import json


def main() -> int:
    target = ROOT / "docs" / "_generated" / "ops-actions.md"
    surface = json.loads((ROOT / "ops/inventory/surfaces.json").read_text(encoding="utf-8"))
    actions = sorted(str(x) for x in surface.get("entrypoints", []) if isinstance(x, str))
    lines = [
        "# Ops Actions",
        "",
        "Generated from `atlasctl ops --list-actions --json`.",
        "",
        f"- total: {len(actions)}",
        "",
        "## Actions",
        "",
    ]
    for action in actions:
        lines.append(f"- `{action}`")
    lines.append("")
    expected = "\n".join(lines)
    actual = target.read_text(encoding="utf-8") if target.exists() else ""
    if actual != expected:
        print("ops actions generated docs drift: docs/_generated/ops-actions.md", file=sys.stderr)
        print("- run: python3 packages/atlasctl/src/atlasctl/commands/ops/runtime_modules/actions_inventory.py > /tmp/ops-actions.json", file=sys.stderr)
        print("- regenerate docs/_generated/ops-actions.md from actions_inventory.render_ops_actions_doc(...)", file=sys.stderr)
        return 1
    print("ops actions generated docs check passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
