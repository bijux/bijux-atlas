from __future__ import annotations

import re
from pathlib import Path


_SRC_ROOT = "packages/atlasctl/src/atlasctl"
_PLACEHOLDER_STEM = re.compile(r"(?:^|[_-])(part\d+|chunk[_-]?\d+|placeholder|tmp)(?:$|[_-])", re.IGNORECASE)


def _package_dirs(src_root: Path) -> list[Path]:
    return sorted(
        path
        for path in src_root.rglob("*")
        if path.is_dir() and (path / "__init__.py").exists() and "__pycache__" not in path.as_posix()
    )


def check_no_empty_packages(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / _SRC_ROOT
    offenders: list[str] = []
    for directory in _package_dirs(src_root):
        rel = directory.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        py_files = [p for p in directory.glob("*.py") if p.name != "__init__.py"]
        if py_files:
            continue
        if (directory / "README.md").exists():
            continue
        offenders.append(rel)
    if offenders:
        return 1, [f"empty package missing README.md: {path}" for path in offenders]
    return 0, []


def check_no_placeholder_module_names(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / _SRC_ROOT
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        if _PLACEHOLDER_STEM.search(path.stem):
            offenders.append(rel)
    if offenders:
        return 1, [f"placeholder-like module name is forbidden: {path}" for path in offenders]
    return 0, []


def check_package_has_module_or_readme(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / _SRC_ROOT
    offenders: list[str] = []
    for directory in _package_dirs(src_root):
        rel = directory.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        has_module = any(p.name != "__init__.py" for p in directory.glob("*.py"))
        has_readme = (directory / "README.md").exists()
        if not has_module and not has_readme:
            offenders.append(rel)
    if offenders:
        return 1, [f"package must contain a real module or README.md: {path}" for path in offenders]
    return 0, []
