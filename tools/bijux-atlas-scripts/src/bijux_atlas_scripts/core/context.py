from __future__ import annotations

import os
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path

from .git import read_git_context


@dataclass(frozen=True)
class RunContext:
    run_id: str
    profile: str
    repo_root: Path
    evidence_root: Path
    no_network: bool
    git_sha: str
    git_dirty: bool

    @classmethod
    def from_args(
        cls, run_id: str | None, evidence_root: str | None, profile: str | None, no_network: bool
    ) -> "RunContext":
        repo_root = Path(__file__).resolve().parents[5]
        git_ctx = read_git_context(repo_root)
        default_run = f"atlas-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}-{git_ctx.sha}"
        resolved_run_id = run_id or os.environ.get("RUN_ID", default_run)
        resolved_profile = profile or os.environ.get("PROFILE", "local")
        root = Path(evidence_root or os.environ.get("EVIDENCE_ROOT", "artifacts/evidence"))
        evidence_root_path = (repo_root / root).resolve() if not root.is_absolute() else root.resolve()
        return cls(
            run_id=resolved_run_id,
            profile=resolved_profile,
            repo_root=repo_root,
            evidence_root=evidence_root_path,
            no_network=no_network,
            git_sha=git_ctx.sha,
            git_dirty=git_ctx.is_dirty,
        )
