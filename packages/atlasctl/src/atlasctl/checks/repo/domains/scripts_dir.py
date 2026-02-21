from __future__ import annotations

from pathlib import Path


def check_scripts_dir_absent(repo_root: Path) -> tuple[int, list[str]]:
    errors: list[str] = []

    scripts_dir = repo_root / "scripts"
    if scripts_dir.exists():
        errors.append("forbidden top-level directory exists: scripts/")

    for entry in repo_root.iterdir():
        if not entry.is_symlink():
            continue
        try:
            raw_target = Path(entry.readlink())
        except OSError:
            continue
        target = (entry.parent / raw_target).resolve(strict=False)
        try:
            rel_target = target.relative_to(repo_root).as_posix()
        except ValueError:
            continue
        if rel_target == "scripts" or rel_target.startswith("scripts/"):
            errors.append(f"forbidden top-level symlink points to scripts/: {entry.name} -> {rel_target}")

    return (1 if errors else 0), errors
