from __future__ import annotations

from pathlib import Path


_ALLOWED_PACKAGE_ROOT_ITEMS = {
    "ARCHITECTURE.md",
    "src",
    "tests",
    "docs",
    "pyproject.toml",
    "README.md",
    "LICENSE",
    "requirements.in",
    "requirements.lock.txt",
    "uv.lock",
    ".pytest_cache",
}


def check_required_target_shape(repo_root: Path) -> tuple[int, list[str]]:
    package_root = repo_root / "packages/atlasctl"
    errors: list[str] = []

    current = {p.name for p in package_root.iterdir()}
    unexpected = sorted(current - _ALLOWED_PACKAGE_ROOT_ITEMS)
    if unexpected:
        errors.append(f"unexpected package-root items: {unexpected}")

    symlinks = sorted(
        p.relative_to(repo_root).as_posix()
        for p in (package_root / "src/atlasctl").rglob("*")
        if p.is_symlink()
    )
    if symlinks:
        errors.append(f"symlinks are forbidden under packages/atlasctl/src: {symlinks}")

    return (0 if not errors else 1), errors
