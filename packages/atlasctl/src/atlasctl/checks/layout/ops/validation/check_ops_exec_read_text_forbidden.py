#!/usr/bin/env python3
from __future__ import annotations

import ast
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
OPS = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"


def main() -> int:
    offenders: list[str] = []
    for p in sorted(OPS.rglob("*.py")):
        tree = ast.parse(p.read_text(encoding="utf-8"), filename=str(p))
        for node in ast.walk(tree):
            if not isinstance(node, ast.Call):
                continue
            if not isinstance(node.func, ast.Name) or node.func.id != "exec":
                continue
            if not node.args:
                continue
            found = False
            for sub in ast.walk(node.args[0]):
                if isinstance(sub, ast.Call) and isinstance(sub.func, ast.Attribute) and sub.func.attr == "read_text":
                    found = True
                    break
            if found:
                offenders.append(p.relative_to(ROOT).as_posix())
                break
    if offenders:
        print("ops exec(read_text()) patterns forbidden:", file=sys.stderr)
        for o in offenders:
            print(o, file=sys.stderr)
        return 1
    print("ops exec(read_text()) policy passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
