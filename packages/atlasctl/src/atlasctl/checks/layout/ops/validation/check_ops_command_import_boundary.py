#!/usr/bin/env python3
from __future__ import annotations

import ast
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parents[8]
OPS_DIR = ROOT / "packages/atlasctl/src/atlasctl/commands/ops"
ALLOWED_PREFIXES = (
    "atlasctl.core.",
    "atlasctl.contracts.",
    "atlasctl.reporting.",
    "atlasctl.registry.",
    "atlasctl.commands._shared",
    "atlasctl.commands.ops.",
)
ALLOWED_INTERNAL_FILES = {
    "packages/atlasctl/src/atlasctl/commands/ops/command.py",
    "packages/atlasctl/src/atlasctl/commands/ops/__init__.py",
    "packages/atlasctl/src/atlasctl/commands/ops/_contracts.py",
}
TEMP_GLUE_PREFIXES = (
    "packages/atlasctl/src/atlasctl/commands/ops/load/checks/",
    "packages/atlasctl/src/atlasctl/commands/ops/load/contracts/",
    "packages/atlasctl/src/atlasctl/commands/ops/internal/",
)


def _imports_from_file(path: Path) -> list[str]:
    tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
    names: list[str] = []
    for node in ast.walk(tree):
        if isinstance(node, ast.Import):
            names.extend(alias.name for alias in node.names)
        elif isinstance(node, ast.ImportFrom):
            if node.level > 0:
                continue
            if node.module:
                names.append(node.module)
    return names


def main() -> int:
    offenders: list[str] = []
    for path in sorted(OPS_DIR.rglob("*.py")):
        rel = path.relative_to(ROOT).as_posix()
        if "/internal/" in rel:
            continue
        if rel in ALLOWED_INTERNAL_FILES:
            continue
        if any(rel.startswith(p) for p in TEMP_GLUE_PREFIXES):
            continue
        for name in _imports_from_file(path):
            if name.startswith("atlasctl.cli"):
                offenders.append(f"{rel}: forbidden import {name}")
                continue
            if name.startswith("atlasctl.") and not any(name.startswith(p) for p in ALLOWED_PREFIXES):
                offenders.append(f"{rel}: out-of-boundary import {name}")
    if offenders:
        print("ops command import boundary failed:", file=sys.stderr)
        for line in offenders:
            print(line, file=sys.stderr)
        return 1
    print("ops command import boundary passed")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
