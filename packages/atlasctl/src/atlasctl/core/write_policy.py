from __future__ import annotations

from pathlib import Path

from .context import RunContext

FORBIDDEN_SOURCE_ROOTS = ("ops", "configs", "docs", "makefiles", "crates")


def allowed_write_roots(ctx: RunContext) -> tuple[Path, ...]:
    return (ctx.evidence_root.resolve(),)


def is_forbidden_repo_path(ctx: RunContext, path: Path) -> bool:
    resolved = path.resolve() if path.is_absolute() else (ctx.repo_root / path).resolve()
    for name in FORBIDDEN_SOURCE_ROOTS:
        root = (ctx.repo_root / name).resolve()
        if resolved == root or root in resolved.parents:
            return True
    return False


def symlink_allowed(path: Path) -> bool:
    # Root symlink policy is maintained via configs/repo/symlink-allowlist.json checks.
    return path.exists()
