from __future__ import annotations

from pathlib import Path


def normalize_evidence_paths(repo_root: Path, values: tuple[str, ...]) -> tuple[str, ...]:
    out: set[str] = set()
    for raw in values:
        value = str(raw).strip()
        if not value:
            continue
        path = Path(value)
        if path.is_absolute():
            try:
                out.add(path.relative_to(repo_root).as_posix())
            except ValueError:
                out.add(path.as_posix())
            continue
        out.add(path.as_posix())
    return tuple(sorted(out))


def validate_write_roots(values: tuple[str, ...]) -> tuple[bool, list[str]]:
    errors: list[str] = []
    for raw in values:
        value = str(raw).strip()
        if not value:
            errors.append("write root must be non-empty")
            continue
        if value.startswith("/") or value.startswith(".."):
            errors.append(f"write root must be repo-relative: {value}")
    return (not errors, errors)


__all__ = ["normalize_evidence_paths", "validate_write_roots"]
