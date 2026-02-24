from __future__ import annotations

from pathlib import Path

from .registry import check_tags, list_checks
from .registry_legacy.ssot import generate_registry_json, toml_entry_from_check, write_registry_toml


def generate_registry(repo_root: Path, *, check_only: bool = False) -> tuple[Path, bool]:
    checks = sorted(list_checks(), key=lambda check: str(check.check_id))
    rows = sorted((toml_entry_from_check(check, groups=check_tags(check)) for check in checks), key=lambda row: str(row.get("id", "")))
    write_registry_toml(repo_root, rows)
    return generate_registry_json(repo_root, check_only=check_only)


__all__ = ["generate_registry"]
