from __future__ import annotations

import hashlib
import json
import shutil
import time
from dataclasses import dataclass
from pathlib import Path
from typing import Iterable

from atlasctl.core.context import RunContext
from atlasctl.core.process import run_command


@dataclass(frozen=True)
class ToolInvocationResult:
    tool: str
    cmd: list[str]
    code: int
    stdout: str
    stderr: str
    combined_output: str
    started_at: float
    ended_at: float

    @property
    def duration_ms(self) -> int:
        return int(round((self.ended_at - self.started_at) * 1000))


def preflight_tools(required: Iterable[str]) -> tuple[list[str], dict[str, str]]:
    missing: list[str] = []
    resolved: dict[str, str] = {}
    for tool in required:
        path = shutil.which(tool)
        if path is None:
            missing.append(tool)
        else:
            resolved[tool] = path
    return missing, resolved


def run_tool(ctx: RunContext, cmd: list[str]) -> ToolInvocationResult:
    started = time.time()
    result = run_command(cmd, ctx.repo_root, ctx=ctx)
    ended = time.time()
    return ToolInvocationResult(
        tool=cmd[0] if cmd else "",
        cmd=cmd,
        code=result.code,
        stdout=result.stdout,
        stderr=result.stderr,
        combined_output=result.combined_output,
        started_at=started,
        ended_at=ended,
    )


def command_rendered(cmd: list[str]) -> str:
    return " ".join(cmd)


def hash_inputs(repo_root: Path, paths: Iterable[str]) -> str:
    h = hashlib.sha256()
    for rel in sorted(set(paths)):
        p = (repo_root / rel).resolve()
        h.update(rel.encode("utf-8"))
        if p.exists() and p.is_file():
            h.update(p.read_bytes())
        else:
            h.update(b"<missing>")
    return h.hexdigest()


def environment_summary(ctx: RunContext, tools: Iterable[str]) -> dict[str, object]:
    missing, resolved = preflight_tools(tools)
    return {
        "required_tools": sorted(set(tools)),
        "missing_tools": sorted(missing),
        "resolved_paths": {k: resolved[k] for k in sorted(resolved)},
        "run_id": ctx.run_id,
    }


def invocation_report(result: ToolInvocationResult) -> dict[str, object]:
    return {
        "tool": result.tool,
        "command_rendered": command_rendered(result.cmd),
        "timings": {
            "start_unix_s": result.started_at,
            "end_unix_s": result.ended_at,
            "duration_ms": result.duration_ms,
        },
        "exit_code": result.code,
        "stdout": result.stdout,
        "stderr": result.stderr,
    }
