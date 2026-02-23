from __future__ import annotations

import hashlib
import json
import os
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


def _matches_prefix(cmd: list[str], prefix: list[str]) -> bool:
    if len(cmd) < len(prefix):
        return False
    return cmd[: len(prefix)] == prefix


def _load_network_policy(ctx: RunContext) -> dict[str, object]:
    path = ctx.repo_root / "configs" / "ops" / "network-policy.json"
    try:
        return json.loads(path.read_text(encoding="utf-8"))
    except Exception:
        return {"schema_version": 1, "default_mode": "allow"}


def _network_policy_forbids(ctx: RunContext, cmd: list[str]) -> tuple[bool, str]:
    policy = _load_network_policy(ctx)
    env_cfg = policy.get("forbid_when", {}) if isinstance(policy, dict) else {}
    names = env_cfg.get("env_any", []) if isinstance(env_cfg, dict) else []
    true_vals = {str(v).strip().lower() for v in (env_cfg.get("env_value_true", []) if isinstance(env_cfg, dict) else [])}
    forbid = bool(ctx.no_network)
    for name in names if isinstance(names, list) else []:
        if str(os.environ.get(str(name), "")).strip().lower() in true_vals:
            forbid = True
            break
    if not forbid:
        return False, ""
    network_inv = policy.get("network_invocations", []) if isinstance(policy, dict) else []
    allow_inv = policy.get("allow_when_forbidden", []) if isinstance(policy, dict) else []
    cmd_strs = [str(x) for x in cmd]
    is_networky = any(_matches_prefix(cmd_strs, [str(x) for x in row]) for row in network_inv if isinstance(row, list))
    is_allowed = any(_matches_prefix(cmd_strs, [str(x) for x in row]) for row in allow_inv if isinstance(row, list))
    if is_networky and not is_allowed:
        return True, "network policy forbids this external invocation in current lane"
    return False, ""


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
    blocked, reason = _network_policy_forbids(ctx, cmd)
    if blocked:
        ended = time.time()
        return ToolInvocationResult(
            tool=cmd[0] if cmd else "",
            cmd=cmd,
            code=2,
            stdout="",
            stderr=reason,
            combined_output=reason,
            started_at=started,
            ended_at=ended,
        )
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
