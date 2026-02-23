#!/usr/bin/env python3
from __future__ import annotations

import ast
import json
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
OPS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
ALLOWLIST = ROOT / "configs/ops/ops-subprocess-allowlist.json"


def _allowed(rel: str, prefixes: list[str]) -> bool:
    return any(rel.startswith(prefix) for prefix in prefixes)


def main() -> int:
    prefixes = json.loads(ALLOWLIST.read_text(encoding="utf-8")).get("paths", [])
    offenders: list[str] = []
    for p in sorted(OPS.rglob("*.py")):
        rel = p.relative_to(ROOT).as_posix()
        if "/internal/" in rel:
            continue
        tree = ast.parse(p.read_text(encoding="utf-8"), filename=str(p))
        imported_subprocess = False
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                if any(alias.name == "subprocess" for alias in node.names):
                    imported_subprocess = True
            elif isinstance(node, ast.ImportFrom) and node.module == "subprocess":
                imported_subprocess = True
        if imported_subprocess and not _allowed(rel, list(prefixes)):
            offenders.append(rel)
    if offenders:
        print("ops subprocess boundary failed:", file=sys.stderr)
        for o in offenders:
            print(o, file=sys.stderr)
        return 1
    print("ops subprocess boundary passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
