#!/usr/bin/env python3
from __future__ import annotations

import json
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
CHECKS_ROOT = ROOT / "packages/atlasctl/src/atlasctl/checks"
ALLOW_CFG = ROOT / "configs/policy/checks-shell-direct-allowlist.json"
TOKENS = ("subprocess.", "run_command(", "shell_script_command(")


def main() -> int:
    allow = json.loads(ALLOW_CFG.read_text(encoding="utf-8"))
    allowed_paths = {str(x) for x in allow.get("paths", [])}
    errs: list[str] = []
    seen_direct: set[str] = set()
    for path in CHECKS_ROOT.rglob("*.py"):
        rel = path.relative_to(ROOT).as_posix()
        text = path.read_text(encoding="utf-8", errors="ignore")
        if any(tok in text for tok in TOKENS):
            seen_direct.add(rel)
            if rel not in allowed_paths:
                errs.append(f"{rel}: direct shell/process execution in checks forbidden unless allowlisted")
    for rel in sorted(allowed_paths - seen_direct):
        errs.append(f"stale checks-shell allowlist entry: {rel}")
    if errs:
        print("\n".join(errs))
        return 1
    print("checks shell boundary OK")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
