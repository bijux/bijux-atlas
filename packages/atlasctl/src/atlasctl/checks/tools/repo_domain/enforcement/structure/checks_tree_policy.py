from __future__ import annotations

from pathlib import Path


_CHECKS_ROOT = Path("packages/atlasctl/src/atlasctl/checks")
_MAX_DEPTH = 10
_MAX_PY_FILES_PER_DIR = 15
_FORBIDDEN_DIR_NAMES = {"misc", "utils"}


def check_checks_tree_policy(repo_root: Path) -> tuple[int, list[str]]:
    checks_root = repo_root / _CHECKS_ROOT
    if not checks_root.exists():
        return 1, [f"missing checks root: {_CHECKS_ROOT.as_posix()}"]

    errors: list[str] = []
    for path in sorted(checks_root.rglob("*")):
        if not path.is_dir():
            continue
        if "__pycache__" in path.parts:
            continue
        rel = path.relative_to(repo_root).as_posix()
        parts = rel.split("/")
        depth = len(parts) - 1
        if depth > _MAX_DEPTH:
            errors.append(f"{rel}: checks tree depth exceeds budget ({depth} > {_MAX_DEPTH})")

        name = path.name
        if name in _FORBIDDEN_DIR_NAMES:
            errors.append(f"{rel}: forbidden directory name `{name}` in checks tree")

        for idx in range(len(parts) - 1):
            if parts[idx] == parts[idx + 1]:
                errors.append(f"{rel}: duplicate adjacent area segment `{parts[idx]}`")
                break

        py_files = [item for item in path.iterdir() if item.is_file() and item.suffix == ".py"]
        if len(py_files) > _MAX_PY_FILES_PER_DIR:
            errors.append(
                f"{rel}: python files per directory exceed budget ({len(py_files)} > {_MAX_PY_FILES_PER_DIR})"
            )
    return (0 if not errors else 1), errors
