"""Shared datatypes for runtime context and path configuration."""

from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from .context import RunContext


@dataclass(frozen=True)
class EvidenceConfig:
    evidence_root: Path
    run_dir: Path
    scripts_artifact_root: Path


@dataclass(frozen=True)
class PathConfig:
    repo_root: Path
    configs_root: Path
    docs_root: Path
    ops_root: Path


def build_path_config(repo_root: Path) -> PathConfig:
    return PathConfig(
        repo_root=repo_root,
        configs_root=repo_root / "configs",
        docs_root=repo_root / "docs",
        ops_root=repo_root / "ops",
    )


def build_evidence_config(ctx: RunContext) -> EvidenceConfig:
    return EvidenceConfig(
        evidence_root=ctx.evidence_root,
        run_dir=ctx.run_dir,
        scripts_artifact_root=ctx.scripts_artifact_root,
    )
