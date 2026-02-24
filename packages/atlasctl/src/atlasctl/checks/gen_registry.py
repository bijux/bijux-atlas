from __future__ import annotations

from pathlib import Path

from .registry import list_checks
from .registry import generate_registry_json


def generate_registry(repo_root: Path, *, check_only: bool = False) -> tuple[Path, bool]:
    # Keep deterministic ordering warm-up aligned with registry generator expectations.
    sorted(list_checks(), key=lambda check: str(check.check_id))
    return generate_registry_json(repo_root, check_only=check_only)


__all__ = ["generate_registry"]
