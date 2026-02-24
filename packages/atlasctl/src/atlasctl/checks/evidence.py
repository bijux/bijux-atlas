from __future__ import annotations

from pathlib import Path

DEFAULT_WRITE_ROOT = "artifacts/evidence"


def ensure_explicit_repo_root(repo_root: Path) -> tuple[bool, list[str]]:
    if str(repo_root).strip() in {"", "."}:
        return False, ["repo_root must be explicit and not the current-directory path"]
    if not repo_root.is_absolute():
        return False, [f"repo_root must be absolute: {repo_root}"]
    return True, []


def validate_within_roots(repo_root: Path, rel_path: str, allowed_roots: tuple[str, ...]) -> tuple[bool, str]:
    normalized = str(rel_path).strip()
    if not normalized:
        return False, "path must be non-empty"
    path = Path(normalized)
    if path.is_absolute():
        try:
            path = path.relative_to(repo_root)
        except ValueError:
            return False, f"path outside repo_root: {normalized}"
    if str(path) in {".", ""}:
        return False, "path must not be repository root"
    if str(path).startswith(".."):
        return False, f"path escapes repo root: {normalized}"
    for root in allowed_roots:
        base = Path(root)
        if path == base or base in path.parents:
            return True, ""
    return False, f"path outside allowed roots: {normalized}"


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


def validate_evidence_paths(repo_root: Path, values: tuple[str, ...], *, allowed_roots: tuple[str, ...]) -> tuple[bool, list[str]]:
    errors: list[str] = []
    normalized = normalize_evidence_paths(repo_root, values)
    for item in normalized:
        ok, reason = validate_within_roots(repo_root, item, allowed_roots)
        if not ok:
            errors.append(reason)
    return (not errors, errors)


__all__ = [
    "DEFAULT_WRITE_ROOT",
    "ensure_explicit_repo_root",
    "normalize_evidence_paths",
    "validate_evidence_paths",
    "validate_within_roots",
    "validate_write_roots",
]
