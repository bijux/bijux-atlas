from __future__ import annotations

from pathlib import Path

MAX_MODULE_LOC = 600


def check_module_size(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for py in sorted(root.rglob("*.py")):
        rel = py.relative_to(repo_root).as_posix()
        if "/legacy/" in rel:
            continue
        loc = sum(1 for _ in py.open("r", encoding="utf-8"))
        if loc > MAX_MODULE_LOC:
            offenders.append(f"{rel}: {loc} LOC > {MAX_MODULE_LOC}")
    if offenders:
        return 1, offenders
    return 0, []
