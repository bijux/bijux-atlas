from __future__ import annotations

import os
from dataclasses import dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Literal

from .git import read_git_context

OutputFormat = Literal["text", "json"]
NetworkMode = Literal["allow", "forbid"]


@dataclass(frozen=True)
class RunContext:
    run_id: str
    profile: str
    repo_root: Path
    evidence_root: Path
    run_dir: Path
    output_format: OutputFormat
    network_mode: NetworkMode
    verbose: bool
    quiet: bool
    require_clean_git: bool
    no_network: bool
    git_sha: str
    git_dirty: bool

    @classmethod
    def from_args(
        cls,
        run_id: str | None,
        evidence_root: str | None,
        profile: str | None,
        no_network: bool,
        output_format: OutputFormat = "text",
        network_mode: NetworkMode = "allow",
        run_dir: str | None = None,
        verbose: bool = False,
        quiet: bool = False,
        require_clean_git: bool = False,
    ) -> "RunContext":
        repo_root = Path(__file__).resolve().parents[5]
        git_ctx = read_git_context(repo_root)
        default_run = f"atlas-{datetime.now(timezone.utc).strftime('%Y%m%d-%H%M%S')}-{git_ctx.sha}"
        resolved_run_id = run_id or os.environ.get("RUN_ID", default_run)
        resolved_profile = profile or os.environ.get("PROFILE", "local")
        root = Path(evidence_root or os.environ.get("EVIDENCE_ROOT", "artifacts/evidence"))
        evidence_root_path = (repo_root / root).resolve() if not root.is_absolute() else root.resolve()
        run_dir_root = (
            (repo_root / Path(run_dir)).resolve()
            if run_dir and not Path(run_dir).is_absolute()
            else Path(run_dir).resolve()
            if run_dir
            else evidence_root_path
        )
        resolved_network_mode: NetworkMode = "forbid" if no_network else network_mode
        resolved_no_network = resolved_network_mode == "forbid"
        return cls(
            run_id=resolved_run_id,
            profile=resolved_profile,
            repo_root=repo_root,
            evidence_root=evidence_root_path,
            run_dir=run_dir_root / resolved_run_id,
            output_format=output_format,
            network_mode=resolved_network_mode,
            verbose=verbose,
            quiet=quiet,
            require_clean_git=require_clean_git,
            no_network=resolved_no_network,
            git_sha=git_ctx.sha,
            git_dirty=git_ctx.is_dirty,
        )
