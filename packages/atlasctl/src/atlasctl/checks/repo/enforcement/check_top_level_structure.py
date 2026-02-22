from __future__ import annotations

from pathlib import Path

INTENDED = {
    "cli",
    "commands",
    "checks",
    "core",
    "contracts",
    "registry",
    "suite",
    "reporting",
}
TRANSITION_ALLOW = {"__pycache__"}
MAX_TREE_DEPTH = 6
MAX_PY_FILES_PER_DIR = 30


def check_top_level_structure(repo_root: Path) -> tuple[int, list[str]]:
    root = repo_root / "packages/atlasctl/src/atlasctl"
    if not root.exists():
        return 1, ["atlasctl source root missing"]
    dirs = sorted(p.name for p in root.iterdir() if p.is_dir())
    errors: list[str] = []
    unknown = [name for name in dirs if name not in INTENDED and name not in TRANSITION_ALLOW]
    if unknown:
        errors.append(f"unexpected top-level atlasctl packages: {', '.join(unknown)}")
    counted = [name for name in dirs if name not in TRANSITION_ALLOW]
    if len(counted) > 10:
        errors.append("top-level package budget exceeded (>10) excluding transition allowlist")
    missing = sorted(name for name in INTENDED if name not in dirs)
    if missing:
        errors.append(f"missing intended top-level atlasctl packages: {', '.join(missing)}")

    for path in sorted(root.rglob("*")):
        if not path.is_dir() or "__pycache__" in path.parts:
            continue
        depth = len(path.relative_to(root).parts)
        if depth > MAX_TREE_DEPTH:
            errors.append(
                f"{path.relative_to(repo_root).as_posix()}: depth {depth} > {MAX_TREE_DEPTH}",
            )
        py_files = sum(1 for child in path.iterdir() if child.is_file() and child.suffix == ".py")
        if py_files > MAX_PY_FILES_PER_DIR:
            errors.append(
                f"{path.relative_to(repo_root).as_posix()}: python file budget exceeded ({py_files} > {MAX_PY_FILES_PER_DIR})",
            )
    return (0 if not errors else 1), errors
