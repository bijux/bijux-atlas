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
