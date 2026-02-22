from __future__ import annotations

from pathlib import Path

from .repo_root import find_repo_root


def evidence_root_path(repo_root: Path, configured: str | None) -> Path:
    raw = Path(configured or "artifacts/evidence")
    return (repo_root / raw).resolve() if not raw.is_absolute() else raw.resolve()


def scripts_artifact_root_path(repo_root: Path, configured: str | None) -> Path:
    raw = Path(configured or "artifacts/atlasctl/run/default")
    return (repo_root / raw).resolve() if not raw.is_absolute() else raw.resolve()


def run_dir_root_path(repo_root: Path, evidence_root: Path, configured: str | None) -> Path:
    if not configured:
        return evidence_root
    raw = Path(configured)
    return (repo_root / raw).resolve() if not raw.is_absolute() else raw.resolve()


def write_text_file(path: Path, content: str, *, encoding: str = "utf-8") -> Path:
    path.parent.mkdir(parents=True, exist_ok=True)
    path.write_text(content, encoding=encoding)
    return path
