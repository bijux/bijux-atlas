from __future__ import annotations

from pathlib import Path


def check_legacy_package_absent(repo_root: Path) -> tuple[int, list[str]]:
    legacy_root = repo_root / "packages/atlasctl/src/atlasctl/legacy"
    if not legacy_root.exists():
        return 0, []
    files = [p.relative_to(repo_root).as_posix() for p in legacy_root.rglob("*") if p.is_file()]
    if not files:
        return 0, []
    return 1, [
        "pre-1.0 policy violation: atlasctl/legacy must be empty or absent",
        *sorted(files),
    ]
