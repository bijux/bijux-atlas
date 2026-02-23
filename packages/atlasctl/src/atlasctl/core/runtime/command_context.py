from __future__ import annotations

from dataclasses import dataclass
from pathlib import Path

from ..context import RunContext


@dataclass(frozen=True)
class CommandContext:
    run_id: str
    repo_root: Path
    profile: str
    evidence_root: Path
    scripts_artifact_root: Path
    run_dir: Path
    output_format: str
    network_mode: str
    quiet: bool
    verbose: bool

    @property
    def write_roots(self) -> tuple[Path, ...]:
        return (self.evidence_root, self.scripts_artifact_root, self.run_dir)

    @classmethod
    def from_run_context(cls, ctx: RunContext) -> "CommandContext":
        return cls(
            run_id=ctx.run_id,
            repo_root=ctx.repo_root,
            profile=ctx.profile,
            evidence_root=ctx.evidence_root,
            scripts_artifact_root=ctx.scripts_artifact_root,
            run_dir=ctx.run_dir,
            output_format=ctx.output_format,
            network_mode=ctx.network_mode,
            quiet=ctx.quiet,
            verbose=ctx.verbose,
        )
