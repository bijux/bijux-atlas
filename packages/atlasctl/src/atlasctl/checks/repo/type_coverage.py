from __future__ import annotations

import ast
from pathlib import Path


MIN_TYPE_COVERAGE = 0.60
TARGET_DIRS = ("packages/atlasctl/src/atlasctl/core", "packages/atlasctl/src/atlasctl/contracts")


def _iter_python_files(root: Path) -> list[Path]:
    return [p for p in sorted(root.rglob("*.py")) if p.name != "__init__.py"]


def _function_is_typed(node: ast.FunctionDef | ast.AsyncFunctionDef) -> bool:
    if node.returns is None:
        return False
    args = [*node.args.posonlyargs, *node.args.args, *node.args.kwonlyargs]
    if node.args.vararg is not None:
        args.append(node.args.vararg)
    if node.args.kwarg is not None:
        args.append(node.args.kwarg)
    return all(arg.annotation is not None for arg in args)


def _coverage_for_file(path: Path) -> tuple[int, int]:
    module = ast.parse(path.read_text(encoding="utf-8"))
    total = 0
    typed = 0
    for node in ast.walk(module):
        if isinstance(node, (ast.FunctionDef, ast.AsyncFunctionDef)):
            if node.name.startswith("_"):
                continue
            total += 1
            if _function_is_typed(node):
                typed += 1
    return typed, total


def check_type_coverage(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []
    for rel_dir in TARGET_DIRS:
        root = repo_root / rel_dir
        typed = 0
        total = 0
        for path in _iter_python_files(root):
            t, n = _coverage_for_file(path)
            typed += t
            total += n
        ratio = (typed / total) if total else 1.0
        if ratio < MIN_TYPE_COVERAGE:
            errors.append(f"{rel_dir}: type coverage {ratio:.1%} below required {MIN_TYPE_COVERAGE:.0%}")
    return (0 if not errors else 1), errors
