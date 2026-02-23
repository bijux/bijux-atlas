from __future__ import annotations

from dataclasses import dataclass

from atlasctl.core.context import RunContext
from atlasctl.core.process import CommandResult, run_command


@dataclass(frozen=True)
class CliAdapter:
    bin_name: str

    def run(self, ctx: RunContext, *args: str) -> CommandResult:
        return run_command([self.bin_name, *[str(a) for a in args]], ctx.repo_root, ctx=ctx)
