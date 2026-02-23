#!/usr/bin/env python3
from __future__ import annotations

import ast
from pathlib import Path

ROOT = Path(__file__).resolve().parents[7]
CMDS = ROOT / "packages/atlasctl/src/atlasctl/commands"


def _imports(path: Path) -> list[str]:
    tree = ast.parse(path.read_text(encoding="utf-8", errors="ignore"))
    out: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            out.extend(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            mod = node.module or ""
            out.append(mod)
    return out


def main() -> int:
    errs: list[str] = []
    for path in CMDS.rglob('*.py'):
        rel = path.relative_to(ROOT).as_posix()
        for imp in _imports(path):
            if '.tests' in imp or '.fixtures' in imp:
                errs.append(f"{rel}: forbidden import from tests/fixtures (`{imp}`)")
    if errs:
        print('\n'.join(sorted(errs)))
        return 1
    print('commands import boundary OK: no ops tests/fixtures imports')
    return 0

if __name__ == '__main__':
    raise SystemExit(main())
