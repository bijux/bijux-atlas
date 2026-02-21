from __future__ import annotations

import ast
from pathlib import Path

ROOT = Path(__file__).resolve().parents[2]
CORE_ROOT = ROOT / "src" / "atlasctl" / "core"


def _forbidden_imports() -> list[str]:
    violations: list[str] = []
    for path in sorted(CORE_ROOT.rglob("*.py")):
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(ROOT / "src")
        tree = ast.parse(path.read_text(encoding="utf-8"), filename=str(path))
        for node in ast.walk(tree):
            if isinstance(node, ast.Import):
                names = [alias.name for alias in node.names]
            elif isinstance(node, ast.ImportFrom):
                base = node.module or ""
                names = [base]
            else:
                continue
            for name in names:
                if not name:
                    continue
                normalized = name.lstrip(".")
                if normalized.startswith("atlasctl.commands") or normalized.startswith("atlasctl.cli"):
                    violations.append(f"{rel}: forbidden import {name}")
                if normalized.startswith("atlasctl.checks"):
                    violations.append(f"{rel}: forbidden import {name}")
    return violations


def test_core_does_not_import_cli_or_commands() -> None:
    errors = [line for line in _forbidden_imports() if "commands" in line or "cli" in line]
    assert not errors, "\n".join(errors)


def test_core_does_not_import_checks() -> None:
    errors = [line for line in _forbidden_imports() if "checks" in line]
    assert not errors, "\n".join(errors)
