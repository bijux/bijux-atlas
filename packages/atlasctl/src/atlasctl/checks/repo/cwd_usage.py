from __future__ import annotations

from pathlib import Path

_ALLOWED = {"packages/atlasctl/src/atlasctl/core/repo_root.py"}


def check_no_path_cwd_usage(repo_root: Path) -> tuple[int, list[str]]:
    offenders: list[str] = []
    root = repo_root / "packages/atlasctl/src/atlasctl"
    for path in sorted(root.rglob("*.py")):
        rel = path.relative_to(repo_root).as_posix()
        if rel in _ALLOWED:
            continue
        text = path.read_text(encoding="utf-8", errors="ignore")
        if "Path.cwd(" in text:
            offenders.append(rel)
    if offenders:
        return 1, ["Path.cwd() is forbidden outside core/repo_root.py", *offenders]
    return 0, []
