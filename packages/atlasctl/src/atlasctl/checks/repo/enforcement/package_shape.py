from __future__ import annotations

from pathlib import Path


def check_no_nested_same_name_packages(repo_root: Path) -> tuple[int, list[str]]:
    src_root = repo_root / "packages/atlasctl/src/atlasctl"
    offenders: list[str] = []
    for path in sorted(src_root.rglob("*")):
        if not path.is_dir():
            continue
        parts = path.relative_to(src_root).parts
        for left, right in zip(parts, parts[1:]):
            if left == right:
                offenders.append(path.relative_to(repo_root).as_posix())
                break
    if offenders:
        return 1, [f"nested same-name package segment is forbidden: {item}" for item in offenders]
    return 0, []

