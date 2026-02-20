#!/usr/bin/env python3
# Purpose: generate docs/_generated/env-vars.md from configs/contracts/env.schema.json.
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "configs" / "contracts" / "env.schema.json"
OUT = ROOT / "docs" / "_generated" / "env-vars.md"


def main() -> int:
    payload = json.loads(CONTRACT.read_text(encoding="utf-8"))
    keys = sorted(payload.get("allowed_env", []))
    lines = [
        "# Env Vars (Generated)",
        "",
        "Generated from `configs/contracts/env.schema.json`. Do not edit manually.",
        "",
        f"- Count: `{len(keys)}`",
        f"- Enforced prefixes: `{', '.join(payload.get('enforced_prefixes', []))}`",
        f"- Dev escape hatch: `{payload.get('dev_mode_allow_unknown_env', '')}`",
        "",
        "## Allowed Env Vars",
        "",
    ]
    lines.extend(f"- `{k}`" for k in keys)
    OUT.parent.mkdir(parents=True, exist_ok=True)
    OUT.write_text("\n".join(lines) + "\n", encoding="utf-8")
    print(OUT.relative_to(ROOT))
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
